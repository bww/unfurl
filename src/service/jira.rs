use std::env;

use reqwest;
use serde::{Serialize, Deserialize};
use serde_yaml;
use serde_json;

use crate::error;
use crate::config;
use crate::route;
use crate::fetch;
use crate::service;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
  header: Option<String>,
}

impl Config {
  fn new() -> Self {
    Self{
      header: None,
    }
  }

  fn from(conf: Option<&serde_yaml::Value>) -> Result<Config, error::Error> {
    Ok(match conf {
      Some(conf) => serde_yaml::from_value(conf.clone())?,
      None       => Config::new(),
    })
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueFields {
  summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
  key: String,
  fields: IssueFields,
}

pub struct Jira{
  client: reqwest::Client,
  config: Config,
  host: String,

  pattern_issue: route::Pattern,
}

impl Jira {
  pub fn new_with_host(conf: &config::Config, host: &str) -> Result<Jira, error::Error> {
    Ok(Jira{
      client: reqwest::Client::new(),
      config: Config::from(conf.get(service::DOMAIN_JIRA))?,
      host: host.to_string(),
      pattern_issue: route::Pattern::new("/browse/{handle}"),
    })
  }

  fn get(&self, url: &str) -> reqwest::RequestBuilder {
    let mut builder = self.client.get(url)
      .header("Content-Type", "application/json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION));
    if let Some(header) = &self.config.header {
      builder = builder.header("Authorization", header);
    }
    builder
  }

  fn request_issue(&self, _conf: &config::Config, _link: &url::Url, mat: route::Match) -> Result<reqwest::RequestBuilder, error::Error> {
    Ok(self.get(&format!("https://{}/rest/api/3/issue/{}",
      &self.host,
      mat.get("handle").ok_or(error::Error::UnboundVariable("handle".to_string()))?,
    )))
  }

  fn format_issue(&self, _conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    let rsp: Issue = match serde_json::from_slice(data.as_ref()) {
      Ok(rsp)  => rsp,
      Err(err) => return Ok(format!("{} ({})", link, err)),
    };
    Ok(format!("{} (#{})", rsp.fields.summary, rsp.key))
  }
}


impl service::Service for Jira {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    if let Some(mat) = self.pattern_issue.match_path(link.path()) {
      self.request_issue(conf, link, mat)
    }else{
      Err(error::Error::NotFound)
    }
  }

  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    if let Some(_) = self.pattern_issue.match_path(link.path()) {
      self.format_issue(conf, link, rsp)
    }else{
      Err(error::Error::NotFound)
    }
  }
}

