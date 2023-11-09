use std::fs;
use std::io::Read;
use std::env;
use std::path;

use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
  pub auth: serde_yaml::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  services: BTreeMap<String, Service>,
}

impl Config {
  pub fn new() -> Config {
    Config{
      services: BTreeMap::new(),
    }
  }

  pub fn get<'a>(&'a self, domain: &str) -> Option<&'a Service> {
    self.services.get(domain)
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

