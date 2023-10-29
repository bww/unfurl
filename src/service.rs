use crate::error;

pub trait Service {
  fn unfurl(&self, link: &url::Url) -> Result<String, error::Error>;
}

pub struct Github ();

impl Service for Github {
  fn unfurl(&self, link: &url::Url) -> Result<String, error::Error> {
    Ok(format!("<<{}>> (Github!)", link))
  }
}

