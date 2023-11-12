use std::env;

use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;

use crate::error;
use crate::config;
use crate::route;
use crate::fetch;
use crate::service;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
  number: usize,
  title: String,
}

pub struct Github{
  client: reqwest::Client,
  config: config::Service,

  pattern_pr: route::Pattern,
  pattern_issue: route::Pattern,
}

impl Github {
  pub fn new(conf: &config::Config) -> Result<Github, error::Error> {
    Ok(Github{
      client: reqwest::Client::new(),
      config: config::Service::from(conf.get(service::DOMAIN_GITHUB))?,
      pattern_pr: route::Pattern::new("/{org}/{repo}/pull/{num}"),
      pattern_issue: route::Pattern::new("/{org}/{repo}/issues/{num}"),
    })
  }

  fn get(&self, url: &str) -> reqwest::RequestBuilder {
    let builder = self.client.get(url)
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION));
    self.config.authenticate(builder)
  }

  fn request_pr(&self, _conf: &config::Config, _link: &url::Url, mat: route::Match) -> Result<reqwest::RequestBuilder, error::Error> {
    Ok(self.get(&format!("https://api.github.com/repos/{}/{}/pulls/{}",
      mat.get("org").ok_or(error::Error::UnboundVariable("org".to_string()))?,
      mat.get("repo").ok_or(error::Error::UnboundVariable("repo".to_string()))?,
      mat.get("num").ok_or(error::Error::UnboundVariable("num".to_string()))?,
    )))
  }

  fn format_pr(&self, _conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    let name = "pr";
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Ok(format!("{} [{}]", link, err)),
    };
    match config::parse_format(&self.config.format, name)? {
      Some(f) => {
        let rsp: serde_json::Value = match serde_json::from_slice(data.as_ref()) {
          Ok(rsp)  => rsp,
          Err(err) => return Ok(format!("{} [{}]", link, err)),
        };
        Ok(f.render(name, &rsp)?)
      },
      None => {
        let rsp: Issue = match serde_json::from_slice(data.as_ref()) {
          Ok(rsp)  => rsp,
          Err(err) => return Ok(format!("{} [{}]", link, err)),
        };
        Ok(format!("{} (#{})", rsp.title, rsp.number))
      },
    }
  }

  fn request_issue(&self, _conf: &config::Config, _link: &url::Url, mat: route::Match) -> Result<reqwest::RequestBuilder, error::Error> {
    Ok(self.get(&format!("https://api.github.com/repos/{}/{}/issues/{}",
      mat.get("org").ok_or(error::Error::UnboundVariable("org".to_string()))?,
      mat.get("repo").ok_or(error::Error::UnboundVariable("repo".to_string()))?,
      mat.get("num").ok_or(error::Error::UnboundVariable("num".to_string()))?,
    )))
  }

  fn format_issue(&self, _conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    let name = "issue";
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Ok(format!("{} [{}]", link, err)),
    };
    match config::parse_format(&self.config.format, name)? {
      Some(f) => {
        let rsp: serde_json::Value = match serde_json::from_slice(data.as_ref()) {
          Ok(rsp)  => rsp,
          Err(err) => return Ok(format!("{} [{}]", link, err)),
        };
        Ok(f.render(name, &rsp)?)
      },
      None => {
        let rsp: Issue = match serde_json::from_slice(data.as_ref()) {
          Ok(rsp)  => rsp,
          Err(err) => return Ok(format!("{} [{}]", link, err)),
        };
        Ok(format!("{} (#{})", rsp.title, rsp.number))
      },
    }
  }
}


impl service::Service for Github {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    if let Some(mat) = self.pattern_pr.match_path(link.path()) {
      self.request_pr(conf, link, mat)
    }else if let Some(mat) = self.pattern_issue.match_path(link.path()) {
      self.request_issue(conf, link, mat)
    }else{
      Err(error::Error::NotFound)
    }
  }

  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    if let Some(_) = self.pattern_pr.match_path(link.path()) {
      self.format_pr(conf, link, rsp)
    }else if let Some(_) = self.pattern_issue.match_path(link.path()) {
      self.format_issue(conf, link, rsp)
    }else{
      Err(error::Error::NotFound)
    }
  }
}


