use std::io::{Read};
use std::fs;
use std::env;

use clap::Parser;

mod error;
mod config;
mod service;

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
    Some(path) => unfurl(opts, fs::File::open(path)?),
    None       => unfurl(opts, std::io::stdin()),
  }
}

fn unfurl<R: Read>(opts: &Options, mut r: R) -> Result<(), error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;
  let mut text: &str = &data;
  while text.len() > 0 {
    text = match text.find("https://") {
      Some(x) => unfurl_url(&text, x)?,
      None    => {
        print!("{}", text);
        &text[0..0]
      },
    };
  }
  Ok(())
}

fn unfurl_url<'a>(data: &'a str, x: usize) -> Result<&'a str, error::Error> {
  print!("{}", &data[..x]);
  let data: &str = &data[x..];
  let (url, rest) = match data.find(char::is_whitespace) {
    Some(y) => (&data[..y], &data[y..]),
    None    => (data, &data[0..0]),
  };

  let url = url::Url::parse(url)?;
  let host = match url.host_str() {
    Some(host) => host,
    None       => {
      println!("{}", url);
      return Ok(rest); // not a supported URL
    },
  };

  let svc: Option<Box<dyn service::Service>> = match host.to_lowercase().as_ref() {
    "github.com" => Some(Box::new(service::Github{})),
    _            => None,
  };
  match svc {
    Some(svc) => println!("{}", svc.unfurl(&url)?),
    None      => println!("{}", url),
  }

  Ok(rest)
}
