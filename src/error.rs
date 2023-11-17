use std::io;
use std::fmt;
use std::str;
use std::string;
use std::sync::mpsc;

#[derive(Debug)]
pub enum Error {
  IOError(io::Error),
  Utf8Error(str::Utf8Error),
  FromUtf8Error(string::FromUtf8Error),
  UrlParseError(url::ParseError),
  YamlParseError(serde_yaml::Error),
  JsonParseError(serde_json::Error),
  ClientError(reqwest::Error),
  RecvError(mpsc::RecvError),
  TemplateError(tinytemplate::error::Error),
  Invalid(String),
  AddrError,
  SendError,
  NotFound,
}

impl From<str::Utf8Error> for Error {
  fn from(err: str::Utf8Error) -> Self {
    Self::Utf8Error(err)
  }
}

impl From<string::FromUtf8Error> for Error {
  fn from(err: string::FromUtf8Error) -> Self {
    Self::FromUtf8Error(err)
  }
}

impl From<url::ParseError> for Error {
  fn from(err: url::ParseError) -> Self {
    Self::UrlParseError(err)
  }
}

impl From<serde_yaml::Error> for Error {
  fn from(err: serde_yaml::Error) -> Self {
    Self::YamlParseError(err)
  }
}

impl From<serde_json::Error> for Error {
  fn from(err: serde_json::Error) -> Self {
    Self::JsonParseError(err)
  }
}

impl From<reqwest::Error> for Error {
  fn from(err: reqwest::Error) -> Self {
    Self::ClientError(err)
  }
}

impl From<mpsc::RecvError> for Error {
  fn from(err: mpsc::RecvError) -> Self {
    Self::RecvError(err)
  }
}

impl From<tinytemplate::error::Error> for Error {
  fn from(err: tinytemplate::error::Error) -> Self {
    Self::TemplateError(err)
  }
}

impl From<addr::error::Error<'_>> for Error {
  fn from(_: addr::error::Error<'_>) -> Self {
    Self::AddrError
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Self::IOError(err)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IOError(err) => err.fmt(f),
      Self::Utf8Error(err) => err.fmt(f),
      Self::FromUtf8Error(err) => err.fmt(f),
      Self::UrlParseError(err) => err.fmt(f),
      Self::YamlParseError(err) => err.fmt(f),
      Self::JsonParseError(err) => err.fmt(f),
      Self::ClientError(err) => err.fmt(f),
      Self::RecvError(err) => err.fmt(f),
      Self::TemplateError(err) => err.fmt(f),
      Self::Invalid(msg) => write!(f, "{}", msg),
      Self::AddrError => write!(f, "Address error"),
      Self::SendError => write!(f, "Send error"),
      Self::NotFound => write!(f, "Not found"),
    }
  }
}

