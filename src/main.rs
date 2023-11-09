use std::io::{Read};
use std::fs;
use std::env;
use std::collections::HashMap;

use clap::Parser;

mod error;
mod config;
mod service;
mod route;
mod fetch;
mod parse;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
  #[clap(long, help="Use the specified configuration")]
  pub config: Option<String>,
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
  let conf = config::load_default()?;
  match &opts.file {
    Some(path) => unfurl(opts, &conf, fs::File::open(path)?),
    None       => unfurl(opts, &conf, std::io::stdin()),
  }
}

fn unfurl<R: Read>(opts: &Options, conf: &config::Config, mut r: R) -> Result<(), error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;

  let svc = fetch::Service::instance();
  // let rx = svc.fetch_urls(vec![
  //   "https://api.ipify.orz".to_string(),
  //   "https://api.ipify.org".to_string(),
  //   "https://api.ipify.org".to_string(),
  //   "https://api.ipify.org".to_string(),
  //   "https://api.ipify.org".to_string()
  // ])?;
  // println!(">>> >>> >>> {:?}", rx.recv()?);

  let mut text: &str = &data;
  let mut toks: Vec<parse::Token> = Vec::new();
  let mut urls: Vec<fetch::Request> = Vec::new();
  loop {
    let (tok, rest) = parse::next(text);
    match tok {
      parse::Token::EOF => break,
      parse::Token::Text(text) => toks.push(tok.clone()),
      parse::Token::URL(text) => match service::find(conf, text)? {
        Some((svc, url)) => {
          urls.push(fetch::Request::new(text, svc.request(conf, &url)?));
          toks.push(tok.clone());
        },
        None => toks.push(parse::Token::Text(text)), // convert to text
      },
    };
    toks.push(tok.clone());
    text = rest;
  }

  let res = svc.fetch_requests(urls)?.recv()?;
  let rsps: HashMap<String, fetch::Response> = res.into_iter()
    .map(|e| { (e.key().to_string(), e) })
    .collect();

  for tok in &toks {
    match tok {
      parse::Token::EOF => break,
      parse::Token::Text(text) => print!("{}", text),
      parse::Token::URL(text) => match service::find(conf, text)? {
        Some((svc, url)) => print!("{}", svc.format(conf, &url, rsps.get(&url.to_string()).expect("No respose for URL"))?),
        None             => print!("{} (INVALID)", text),
      },
    };
  }

  Ok(())
}

// fn unfurl_request<'a>(conf: &config::Config, data: &'a str, x: usize) -> Result<&'a str, error::Error> {
//   print!("{}", &data[..x]);
//   let data: &str = &data[x..];
//   let (url, rest) = match data.find(char::is_whitespace) {
//     Some(y) => (&data[..y], &data[y..]),
//     None    => (data, &data[0..0]),
//   };
// 
//   let url = url::Url::parse(url)?;
//   let host = match url.host_str() {
//     Some(host) => host,
//     None       => {
//       println!("{}", url);
//       return Ok(rest); // not a supported URL
//     },
//   };
// 
//   let svc: Option<Box<dyn service::Service>> = match host.to_lowercase().as_ref() {
//     service::DOMAIN_GITHUB => Some(Box::new(service::Github::new(conf)?)),
//     _                      => None,
//   };
//   match svc {
//     Some(svc) => println!("{}", svc.unfurl(conf, &url)?),
//     None      => println!("{}", url),
//   }
// 
//   Ok(rest)
// }

// fn next_url<'a>(conf: &config::Config, text: &'a str) -> Result<(&'a str, &'a str, error::Error> {
//   let (text, rest) = match text.find("https://") {
//     Some(x) => (&text[..x], &text[x..]),
//     None    => return (text, ""),
//   };
//   let (url, rest) = match rest.find(char::is_whitespace) {
//     Some(y) => (&text[..y], &text[y..]),
//     None    => (data, &data[0..0]),
//   };
// 
//   let url = url::Url::parse(url)?;
//   let host = match url.host_str() {
//     Some(host) => host,
//     None       => ,
//   };
// 
//   let svc: Option<Box<dyn service::Service>> = match host.to_lowercase().as_ref() {
//     service::DOMAIN_GITHUB => Some(Box::new(service::Github::new(conf)?)),
//     _                      => None,
//   };
//   match svc {
//     Some(svc) => println!("{}", svc.unfurl(conf, &url)?),
//     None      => println!("{}", url),
//   }
// 
//   Ok(rest)
// }

// fn unfurl_url<'a>(conf: &config::Config, data: &'a str, x: usize) -> Result<&'a str, error::Error> {
//   print!("{}", &data[..x]);
//   let data: &str = &data[x..];
//   let (url, rest) = match data.find(char::is_whitespace) {
//     Some(y) => (&data[..y], &data[y..]),
//     None    => (data, &data[0..0]),
//   };
// 
//   let url = url::Url::parse(url)?;
//   let host = match url.host_str() {
//     Some(host) => host,
//     None       => {
//       println!("{}", url);
//       return Ok(rest); // not a supported URL
//     },
//   };
// 
//   let svc: Option<Box<dyn service::Service>> = match host.to_lowercase().as_ref() {
//     service::DOMAIN_GITHUB => Some(Box::new(service::Github::new(conf)?)),
//     _                      => None,
//   };
//   match svc {
//     Some(svc) => println!("{}", svc.unfurl(conf, &url)?),
//     None      => println!("{}", url),
//   }
// 
//   Ok(rest)
// }
