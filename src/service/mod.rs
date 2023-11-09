use reqwest;
use addr;

use crate::error;
use crate::config;
use crate::fetch;

mod github;
mod jira;

pub const DOMAIN_GITHUB: &str = "github.com";
pub const DOMAIN_JIRA: &str   = "atlassian.net";

pub trait Service {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error>;
  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error>;
}

pub fn find(conf: &config::Config, url: &str) -> Result<Option<(Box<dyn Service>, url::Url)>, error::Error> {
  let url = match url::Url::parse(url) {
    Ok(url) => url,
    Err(_)  => return Ok(None),
  };
  let host = match url.host_str() {
    Some(host) => host,
    None       => return Ok(None),
  };
  let root = match addr::parse_domain_name(host)?.root() {
    Some(root) => root,
    None       => host, // weird; just use the input host
  };
  match root.to_lowercase().as_ref() {
    DOMAIN_GITHUB => Ok(Some((Box::new(github::Github::new(conf)?), url))),
    DOMAIN_JIRA   => Ok(Some((Box::new(jira::Jira::new_with_host(conf, &host)?), url))),
    _             => Ok(None)
  }
}

