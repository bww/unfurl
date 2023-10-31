use std::env;
use std::path;

use reqwest::blocking;
use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::error;
use crate::config;
use crate::route;

pub const DOMAIN_GITHUB: &str = "github.com";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Service {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GithubConfig {
  header: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {
  number: usize,
  title: String,
}

pub struct Github{
  client: blocking::Client,
  config: GithubConfig,
  pattern_pr: route::Pattern,
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
      pattern_pr: route::Pattern::new("/{org}/{repo}/pull/{num}"),
    })
  }
}


impl Service for Github {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error> {
    let pr = match self.pattern_pr.match_path(link.path()) {
      Some(pr) => pr,
      None     => return Ok(link.to_string()),
    };
    let num = match pr.get("num") {
      Some(num) => num,
      None      => return Ok(link.to_string()),
    };

    let resp: blocking::Response = match self.client.get(&format!("https://api.github.com/repos/treno-io/product/pulls/{}", num))
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION))
      .header("Authorization", &self.config.header)
      .send() {
        Ok(resp) => resp,
        Err(err) => return Ok(format!("{} ({})", link, err)),
    };

    let resp: PullRequest = match resp.json::<PullRequest>() {
      Ok(resp) => resp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };

    Ok(format!("{} (#{})", resp.title, resp.number))
    // let conf = match conf.get(DOMAIN_GITHUB) {
    //   Some(conf) => Ok(format!("<<{}>> (Github! + {:?})", link, conf)),
    //   None       => Err(error::Error::NotFound),
    // }
  }
}

