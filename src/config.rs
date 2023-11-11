use std::fs;
use std::io::Read;
use std::env;
use std::path;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml;
use tinytemplate;

use crate::error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  #[serde(default = "HashMap::new")]
  services: HashMap<String, serde_yaml::Value>,
}

impl Config {
  pub fn new() -> Config {
    Config{
      services: HashMap::new(),
    }
  }

  pub fn get<'a>(&'a self, domain: &str) -> Option<&'a serde_yaml::Value> {
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

pub fn parse_format<'a>(conf: &'a Option<HashMap<String, String>>, name: &'a str) -> Result<Option<tinytemplate::TinyTemplate<'a>>, error::Error> {
  let conf = match conf {
    Some(conf) => conf,
    None       => return Ok(None),
  };
  let tmpl = match conf.get(name) {
    Some(tmpl) => tmpl,
    None       => return Ok(None),
  };
  let mut tt = tinytemplate::TinyTemplate::new();
  tt.add_template(name, tmpl)?;
  Ok(Some(tt))
}


