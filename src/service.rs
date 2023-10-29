use crate::error;
use crate::config;

pub const DOMAIN_GITHUB: &str = "github.com";

pub trait Service {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error>;
}

pub struct Github ();

impl Service for Github {
  fn unfurl(&self, conf: &config::Config, link: &url::Url) -> Result<String, error::Error> {
    match conf.get(DOMAIN_GITHUB) {
      Some(conf) => Ok(format!("<<{}>> (Github! + {:?})", link, conf)),
      None       => Err(error::Error::NotFound),
    }
  }
}

