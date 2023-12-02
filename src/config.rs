use std::fs;
use std::io::Read;
use std::env;
use std::path;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml;
use reqwest;

use crate::error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  #[serde(default = "HashMap::new")]
  services: HashMap<String, Service>,
  #[serde(skip, default = "Service::new")]
  default_service: Service,
}

impl Config {
  pub fn new() -> Config {
    Config{
      services: HashMap::new(),
      default_service: Service::new(),
    }
  }

  pub fn service<'a>(&'a self, domain: &str) -> Option<&'a Service> {
    self.services.get(domain)
  }

  pub fn default_service<'a>(&'a self) -> &'a Service {
    &self.default_service
  }

  pub fn service_or_default<'a>(&'a self, domain: &str) -> &'a Service {
    match self.service(domain) {
      Some(svc) => svc,
      None      => self.default_service(),
    }
  }
}

pub fn load<P: AsRef<path::Path>>(p: &Option<P>) -> Result<Config, error::Error> {
  match p {
    Some(p) => load_data(fs::File::open(p)?),
    None    => load_default(),
  }
}

pub fn load_default() -> Result<Config, error::Error> {
  match env::home_dir() {
    Some(home) => load_data(fs::File::open(home.join(".unfurl.yml"))?),
    None       => Err(error::Error::NotFound),
  }
}

pub fn load_data<R: Read>(mut r: R) -> Result<Config, error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;
  let conf: Config = serde_yaml::from_str(&data)?;
  Ok(conf)
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Authn {
  pub header: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Service {
  pub auth: Option<Authn>,
  pub format: Option<HashMap<String, String>>,
}

pub trait Authenticator {
  fn authenticate(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder;
  fn authenticate_chain<A: Authenticator>(&self, req: reqwest::RequestBuilder, next: Option<A>) -> reqwest::RequestBuilder;
}

impl Service {
  pub fn new() -> Self {
    Self{
      auth: None,
      format: None,
    }
  }

  pub fn format<'a>(&'a self, name: &str) -> Option<&'a str> {
    match &self.format {
      Some(format) => format.get(name).map(|x| x.as_str()),
      None         => None,
    }
  }
}

impl Authenticator for Service {
  fn authenticate(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    self.authenticate_chain::<Service>(req, None)
  }

  fn authenticate_chain<A: Authenticator>(&self, req: reqwest::RequestBuilder, next: Option<A>) -> reqwest::RequestBuilder {
    let auth = match &self.auth {
      Some(auth) => auth,
      None       => return match next {
        Some(next) => next.authenticate(req),
        None       => return req,
      },
    };
    let header = match &auth.header {
      Some(header) => header,
      None       => return match next {
        Some(next) => next.authenticate(req),
        None       => return req,
      },
    };
    req.header("Authorization", header)
  }
}

