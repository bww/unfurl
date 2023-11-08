use std::env;
use std::path;

use reqwest;
use serde::{Serialize, Deserialize};
use serde_yaml;
use serde_json;

use crate::error;
use crate::config;
use crate::route;
use crate::fetch;

pub const DOMAIN_GITHUB: &str = "github.com";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Service {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error>;
  fn format(&self, conf: &config::Config, link: &url::Url, rsp: fetch::Response) -> Result<String, error::Error>;
}

pub fn find(conf: &config::Config, url: &str) -> Result<Option<(Box<dyn Service>, url::Url)>, error::Error> {
  let url = match url::Url::parse(url) {
    Ok(url)  => url,
    Err(err) => return Ok(None),
  };
  let host = match url.host_str() {
    Some(host) => host,
    None       => return Ok(None),
  };
  match host.to_lowercase().as_ref() {
    DOMAIN_GITHUB => Ok(Some((Box::new(Github::new(conf)?), url))),
    _             => Ok(None)
  }
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
  client: reqwest::Client,
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
      client: reqwest::Client::new(),
      config: serde_yaml::from_value(conf.auth.clone())?,
      pattern_pr: route::Pattern::new("/{org}/{repo}/pull/{num}"),
      pattern_issue: route::Pattern::new("/{org}/{repo}/issues/{num}"),
    })
  }

  fn get(&self, url: &str) -> reqwest::RequestBuilder {
    self.client.get(url)
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION))
      .header("Authorization", &self.config.header)
  }

  fn request_pr(&self, conf: &config::Config, link: &url::Url, mat: route::Match) -> Result<reqwest::RequestBuilder, error::Error> {
    match mat.get("num") {
      Some(num) => Ok(self.get(&format!("https://api.github.com/repos/treno-io/product/pulls/{}", num))),
      None      => Err(error::Error::NotFound),
    }
  }

  fn format_pr(&self, conf: &config::Config, link: &url::Url, rsp: fetch::Response) -> Result<String, error::Error> {
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    let rsp: GithubIssue = match serde_json::from_slice(data.as_ref()) {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    Ok(format!("{} (#{})", rsp.title, rsp.number))
  }

  fn request_issue(&self, conf: &config::Config, link: &url::Url, mat: route::Match) -> Result<reqwest::RequestBuilder, error::Error> {
    match mat.get("num") {
      Some(num) => Ok(self.get(&format!("https://api.github.com/repos/treno-io/product/issues/{}", num))),
      None      => Err(error::Error::NotFound),
    }
  }

  fn format_issue(&self, conf: &config::Config, link: &url::Url, rsp: fetch::Response) -> Result<String, error::Error> {
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    let rsp: GithubIssue = match serde_json::from_slice(data.as_ref()) {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    Ok(format!("{} (#{})", rsp.title, rsp.number))
  }
}


impl Service for Github {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    if let Some(mat) = self.pattern_pr.match_path(link.path()) {
      self.request_pr(conf, link, mat)
    }else if let Some(mat) = self.pattern_issue.match_path(link.path()) {
      self.request_issue(conf, link, mat)
    }else{
      Err(error::Error::NotFound)
    }
  }

  fn format(&self, conf: &config::Config, link: &url::Url, rsp: fetch::Response) -> Result<String, error::Error> {
    if let Some(mat) = self.pattern_pr.match_path(link.path()) {
      self.format_pr(conf, link, rsp)
    }else if let Some(mat) = self.pattern_issue.match_path(link.path()) {
      self.format_issue(conf, link, rsp)
    }else{
      Err(error::Error::NotFound)
    }
  }
}

