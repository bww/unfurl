use std::collections::HashMap;

use reqwest;
use addr;

use crate::error;
use crate::config;
use crate::fetch;
use crate::route;

mod github;
mod jira;

pub const DOMAIN_GITHUB: &str = "github.com";
pub const DOMAIN_JIRA: &str   = "atlassian.net";

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

struct Endpoint {
  name: String,
  route: route::Pattern,
  url: String,
  format: String,
}

impl Endpoint {
  fn url<'a>(&'a self) -> &'a str {
    &self.url
  }

  fn url_with_data(&self, mat: &route::Match) -> Result<String, error::Error> {
    let mut f = tinytemplate::TinyTemplate::new();
    f.add_template(&self.name, &self.url)?;
    Ok(f.render(&self.name, &mat.vars)?)
  }
}

pub struct Generic {
  client: reqwest::Client,
  routes: HashMap<String, Vec<Endpoint>>,
}

impl Generic {
  pub fn new() -> Self {
    Self{
      client: reqwest::Client::new(),
      routes: HashMap::from([
        ("github.com".to_string(), vec![
          Endpoint{
            name: "pr".to_string(),
            route: route::Pattern::new("/{org}/{repo}/pull/{num}"),
            url: "https://api.github.com/repos/{org}/{repo}/pulls/{num}".to_string(),
            format: "{title} (#{number})".to_string(),
          },
          Endpoint{
            name: "issue".to_string(),
            route: route::Pattern::new("/{org}/{repo}/issues/{num}"),
            url: "https://api.github.com/repos/{org}/{repo}/issues/{num}".to_string(),
            format: "{title} (#{number})".to_string(),
          },
        ]),
      ]),
    }
  }

  fn find_host<'a>(&'a self, url: &url::Url) -> Option<&'a Vec<Endpoint>> {
    let host = match url.host_str() {
      Some(host) => host,
      None       => return None,
    };
    if let Some(endpoint) = self.routes.get(host) {
      return Some(endpoint);
    }
    let root = match addr::parse_domain_name(host) {
      Ok(addr) => match addr.root() {
        Some(root) => root,
        None       => host, // weird; just use the input host
      },
      Err(_) => return None,
    };
    self.routes.get(root)
  }

  fn find_route<'a>(&'a self, url: &url::Url) -> Option<(&'a Endpoint, route::Match)> {
    let opts = match self.find_host(url) {
      Some(opts) => opts,
      None       => return None,
    };
    for opt in opts {
      if let Some(mat) = opt.route.match_path(url.path()) {
        return Some((opt, mat));
      }
    }
    None
  }

  fn get(&self, url: &str) -> reqwest::RequestBuilder {
    let builder = self.client.get(url)
      .header("Accept", "application/vnd.github+json")
      .header("User-Agent", &format!("Unfurl/{}", VERSION));
    //self.config.authenticate(builder)
    builder
  }
}

impl Service for Generic {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    match self.find_route(link) {
      Some((ept, mat)) => Ok(self.get(&ept.url_with_data(&mat)?)),
      None             => Err(error::Error::NotFound),
    }
  }

  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    Err(error::Error::NotFound)
  }
}

