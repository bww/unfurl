use std::env;
use std::path;

use reqwest::blocking;
use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::error;
use crate::config;

pub const DOMAIN_GITHUB: &str = "github.com";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Service {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GithubConfig {
  header: String,
}

pub struct Github{
  client: blocking::Client,
  config: GithubConfig,
}

impl Github {
  pub fn new(conf: &config::Config) -> Result<Github, error::Error> {
    let conf = match conf.get(DOMAIN_GITHUB) {
      Some(conf) => conf,
      None       => return Err(error::Error::NotFound),
    };
    Ok(Github{
      client: blocking::Client::new(),
      config: serde_yaml::from_value(conf.auth.clone())?,
    })
  }
}

impl Service for Github {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error> {
    let num = match path::Path::new(link.path()).file_name() {
      Some(name) => name.to_string_lossy(),
      None => return Err(error::Error::NotFound),
    };
    let resp = self.client.get(&format!("https://api.github.com/repos/treno-io/product/pulls/{}", num))
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION))
      .header("Authorization", &self.config.header)
      .send()?.text()?;

    println!(">>> * >>> {}", resp);

    Ok(format!("<<{}>> (Github!)", link))
    // let conf = match conf.get(DOMAIN_GITHUB) {
    //   Some(conf) => Ok(format!("<<{}>> (Github! + {:?})", link, conf)),
    //   None       => Err(error::Error::NotFound),
    // }
  }
}

