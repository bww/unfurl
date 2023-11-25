use std::fs;
use std::path;
use std::io::Read;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml;
use reqwest;
use addr;

use crate::error;
use crate::config::{self, Authenticator};
use crate::fetch;
use crate::route;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const BUILTIN_ROUTES: &str = include_str!("../../conf/routes.yml");

pub trait Service {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error>;
  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Endpoint {
  name: String,
  route: route::Pattern,
  url: String,
  format: String,
}

impl Endpoint {
  fn url(&self, link: &url::Url, mat: &route::Match) -> Result<String, error::Error> {
    let cxt = match link.host() {
      Some(host) => mat.vars_with(HashMap::from([("domain".to_string(), host.to_string())])),
      None       => mat.vars.clone(),
    };
    let mut f = tinytemplate::TinyTemplate::new();
    f.add_template(&self.name, &self.url)?;
    Ok(f.render(&self.name, &cxt)?)
  }

  fn format_response(&self, rsp: &fetch::Response) -> Result<String, error::Error> {
    let data = match rsp.data() {
      Ok(data) => data,
      Err(err) => return Err(error::Error::Invalid(format!("Could not read data: {}", err))),
    };
    let rsp: serde_json::Value = match serde_json::from_slice(data.as_ref()) {
      Ok(rsp)  => rsp,
      Err(err) => return Err(error::Error::Invalid(format!("Could not parse data: {}", err))),
    };
    let mut f = tinytemplate::TinyTemplate::new();
    f.add_template(&self.name, &self.format)?;
    Ok(f.render(&self.name, &rsp)?)
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Domain {
  config: Option<config::Service>,
  headers: HashMap<String, String>,
  routes: Vec<Endpoint>,
}

impl Domain {
  fn set_config(&mut self, conf: config::Service) {
    self.config = Some(conf);
  }
}

impl config::Authenticator for &Domain {
  fn authenticate(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    match &self.config {
      Some(conf) => conf.authenticate_chain::<&Domain>(req, None),
      None       => req,
    }
  }

  fn authenticate_chain<A: config::Authenticator>(&self, req: reqwest::RequestBuilder, next: Option<A>) -> reqwest::RequestBuilder {
    match &self.config {
      Some(conf) => conf.authenticate_chain(req, next),
      None       => req,
    }
  }
}

pub struct Generic {
  client: reqwest::Client,
  domains: HashMap<String, Domain>,
}

impl Generic {
  pub fn load_path<P: AsRef<path::Path>>(conf: &config::Config, p: P) -> Result<Self, error::Error> {
    Self::load_data(conf, fs::File::open(p)?)
  }

  pub fn load_default(conf: &config::Config) -> Result<Self, error::Error> {
    Self::load_data(conf, BUILTIN_ROUTES.as_bytes())
  }

  pub fn load_data<R: Read>(conf: &config::Config, mut r: R) -> Result<Self, error::Error> {
    let mut data = String::new();
    r.read_to_string(&mut data)?;
    let mut domains: HashMap<String, Domain> = serde_yaml::from_str(&data)?;
    for (k, v) in domains.iter_mut() {
      v.set_config(config::Service::from(conf.get(k))?);
    }
    Ok(Self{
      client: reqwest::Client::new(),
      domains: domains,
    })
  }

  fn find_host<'a>(&'a self, url: &url::Url) -> Option<&'a Domain> {
    let host = match url.host_str() {
      Some(host) => host,
      None       => return None,
    };
    if let Some(endpoint) = self.domains.get(host) {
      return Some(endpoint);
    }
    let root = match addr::parse_domain_name(host) {
      Ok(addr) => match addr.root() {
        Some(root) => root,
        None       => host, // weird; just use the input host
      },
      Err(_) => return None,
    };
    self.domains.get(root)
  }

  fn find_route<'a>(&'a self, url: &url::Url) -> Option<(&'a Domain, &'a Endpoint, route::Match)> {
    let domain = match self.find_host(url) {
      Some(domain) => domain,
      None         => return None,
    };
    for opt in &domain.routes {
      if let Some(mat) = opt.route.match_path(url.path()) {
        return Some((domain, opt, mat));
      }
    }
    None
  }

  fn get(&self, conf: &config::Service, domain: &Domain, url: &str) -> reqwest::RequestBuilder {
    let mut builder = self.client.get(url)
      .header("User-Agent", &format!("Unfurl/{}", VERSION));
    for (key, val) in &domain.headers {
      builder = builder.header(key, val);
    }
    conf.authenticate_chain(builder, Some(domain))
  }
}

impl Service for Generic {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    let host = match link.host_str() {
      Some(host) => host,
      None       => return Err(error::Error::Invalid("No host".to_string())),
    };
    match self.find_route(link) {
      Some((domain, ept, mat)) => Ok(self.get(&config::Service::from(conf.get(host))?, domain, &ept.url(link, &mat)?)),
      None                     => Err(error::Error::NotFound),
    }
  }

  fn format(&self, _conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error> {
    match self.find_route(link) {
      Some((_, ept, _)) => Ok(ept.format_response(rsp)?),
      None              => Err(error::Error::NotFound),
    }
  }
}

