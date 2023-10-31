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
struct GithubIssue {
  number: usize,
  title: String,
}

pub struct Github{
  client: blocking::Client,
  config: GithubConfig,

  pattern_pr: route::Pattern,
  pattern_issue: route::Pattern,
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
      pattern_issue: route::Pattern::new("/{org}/{repo}/issues/{num}"),
    })
  }

  fn get(&self, url: &str) -> blocking::RequestBuilder {
    self.client.get(url)
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION))
      .header("Authorization", &self.config.header)
  }

  fn unfurl_pr(&self, conf: &config::Config, link: &url::Url, mat: route::Match) -> Result<String, error::Error> {
    let num = match mat.get("num") {
      Some(num) => num,
      None      => return Ok(link.to_string()),
    };
    let rsp: blocking::Response = match self.get(&format!("https://api.github.com/repos/treno-io/product/pulls/{}", num)).send() {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    let rsp: GithubIssue = match rsp.json::<GithubIssue>() {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    Ok(format!("{} (#{})", rsp.title, rsp.number))
  }

  fn unfurl_issue(&self, conf: &config::Config, link: &url::Url, mat: route::Match) -> Result<String, error::Error> {
    let num = match mat.get("num") {
      Some(num) => num,
      None      => return Ok(link.to_string()),
    };
    let rsp: blocking::Response = match self.get(&format!("https://api.github.com/repos/treno-io/product/issues/{}", num)).send() {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    let rsp: GithubIssue = match rsp.json::<GithubIssue>() {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    Ok(format!("{} (#{})", rsp.title, rsp.number))
  }
}


impl Service for Github {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error> {
    if let Some(mat) = self.pattern_pr.match_path(link.path()) {
      self.unfurl_pr(conf, link, mat)
    }else if let Some(mat) = self.pattern_issue.match_path(link.path()) {
      self.unfurl_issue(conf, link, mat)
    }else{
      Ok(link.to_string())
    }
  }
}

