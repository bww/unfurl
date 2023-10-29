use std::fs;
use std::io::Read;
use std::env;

use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
  pub domain: String,
  pub auth: serde_yaml::Mapping,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  pub services: Vec<Service>,
}

pub fn load_default() -> Result<Config, error::Error> {
  match env::home_dir() {
    Some(home) => load(fs::File::open(home.join(".unfurl.yml"))?),
    None       => Err(error::Error::NotFound),
  }
}

pub fn load<R: Read>(mut r: R) -> Result<Config, error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;
  let conf: Config = serde_yaml::from_str(&data)?;
  Ok(conf)
}

