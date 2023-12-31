use std::io::{Read};
use std::fs;
use std::collections::HashMap;

use clap::Parser;

mod error;
mod config;
mod service;
mod route;
mod fetch;
mod parse;

use crate::service::Service;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
  #[clap(long, help="Use the specified configuration")]
  pub config: Option<String>,
  #[clap(long, help="Use the specified routes definition")]
  pub routes: Option<String>,
  #[clap(long, help="Enable debugging mode")]
  pub debug: bool,
  #[clap(long, help="Enable verbose output")]
  pub verbose: bool,
  #[clap(help="Input paths to unfurl")]
  pub file: Option<String>,
}

fn main() {
  let opts = Options::parse();
  match app(&opts) {
    Ok(_)    => {},
    Err(err) => eprintln!("* * * {}", err),
  }
}

fn app(opts: &Options) -> Result<(), error::Error> {
  let conf = match config::load(&opts.config) {
    Ok(conf) => conf,
    Err(err) => match err {
      error::Error::NotFound => config::Config::new(),
      err                    => return Err(err),
    },
  };
  match &opts.file {
    Some(path) => unfurl(opts, &conf, fs::File::open(path)?),
    None       => unfurl(opts, &conf, std::io::stdin()),
  }
}

fn unfurl<R: Read>(opts: &Options, conf: &config::Config, mut r: R) -> Result<(), error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;

  let ftc = fetch::Service::instance();
  let svc = {
    let mut svc = service::Default::load_default(conf)?;
    if let Some(routes) = &opts.routes {
      svc.extend(service::Default::load_path(conf, routes)?);
    }
    svc
  };

  let mut text: &str = &data;
  let mut toks: Vec<parse::Token> = Vec::new();
  let mut urls: Vec<fetch::Request> = Vec::new();
  loop {
    let (tok, rest) = parse::next(text);
    match tok {
      parse::Token::EOF       => break,
      parse::Token::Text(_)   => toks.push(tok.clone()),
      parse::Token::URL(text) => match url::Url::parse(text) {
        Ok(url) => match svc.request(conf, &url) {
          Ok(req) => {
            urls.push(fetch::Request::new(text, req));
            toks.push(tok.clone());
          },
          Err(_) => toks.push(parse::Token::Text(text)), // convert to text
        },
        Err(_) => toks.push(parse::Token::Text(text)), // convert to text
      },
    };
    text = rest;
  }

  let res = ftc.fetch_requests(urls)?.recv()?;
  let rsps: HashMap<String, fetch::Response> = res.into_iter()
    .map(|e| { (e.key().to_string(), e) })
    .collect();

  for tok in &toks {
    match tok {
      parse::Token::EOF        => break,
      parse::Token::Text(text) => print!("{}", text),
      parse::Token::URL(text)  => {
        let url = url::Url::parse(text)?;
        let rsp = rsps.get(&url.to_string()).expect("No respose for URL");
        print!("{}", svc.format(conf, &url, &rsp)?);
      },
    };
  }

  Ok(())
}

