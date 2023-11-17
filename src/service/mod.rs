use std::collections::HashMap;

use reqwest;
use addr;

use crate::error;
use crate::config;
use crate::fetch;
use crate::route;

pub const DOMAIN_GITHUB: &str = "github.com";
pub const DOMAIN_JIRA: &str   = "atlassian.net";

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Service {
  fn request(&self, conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error>;
  fn format(&self, conf: &config::Config, link: &url::Url, rsp: &fetch::Response) -> Result<String, error::Error>;
}

struct Endpoint {
  name: String,
  route: route::Pattern,
  url: String,
  format: String,
}

impl Endpoint {
  fn url(&self, link: &url::Url, mat: &route::Match) -> Result<String, error::Error> {
    let cxt = match link.host_str() {
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
      Err(_)   => return Err(error::Error::Invalid),
    };
    let rsp: serde_json::Value = match serde_json::from_slice(data.as_ref()) {
      Ok(rsp) => rsp,
      Err(_)  => return Err(error::Error::Invalid),
    };
    let mut f = tinytemplate::TinyTemplate::new();
    f.add_template(&self.name, &self.format)?;
    Ok(f.render(&self.name, &rsp)?)
  }
}

struct Domain {
  config: config::Service,
  headers: Vec<(String, String)>,
  routes: Vec<Endpoint>,
}

impl Domain {
  fn authenticate(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    self.config.authenticate(req)
  }
}

pub struct Generic {
  client: reqwest::Client,
  domains: HashMap<String, Domain>,
}

impl Generic {
  pub fn new(conf: &config::Config) -> Result<Self, error::Error> {
    Ok(Self{
      client: reqwest::Client::new(),
      domains: HashMap::from([
        (DOMAIN_GITHUB.to_string(), Domain{
          config: config::Service::from(conf.get(DOMAIN_GITHUB))?,
          headers: vec![
            ("Accept".to_string(), "application/vnd.github+json".to_string())
          ],
          routes: vec![
            Endpoint{
              name: "pr".to_string(),
              route: route::Pattern::new("/{org}/{repo}/pull/{num}"),
              url: "https://api.github.com/repos/{org}/{repo}/pulls/{num}".to_string(),
              format: "{title} (PR #{number})".to_string(),
            },
            Endpoint{
              name: "issue".to_string(),
              route: route::Pattern::new("/{org}/{repo}/issues/{num}"),
              url: "https://api.github.com/repos/{org}/{repo}/issues/{num}".to_string(),
              format: "{title} (Issue #{number})".to_string(),
            },
          ],
        }),
        (DOMAIN_JIRA.to_string(), Domain{
          config: config::Service::from(conf.get(DOMAIN_JIRA))?,
          headers: vec![
            ("Content-Type".to_string(), "application/json".to_string())
          ],
          routes: vec![
            Endpoint{
              name: "issue".to_string(),
              route: route::Pattern::new("/browse/{key}"),
              url: "https://{domain}/rest/api/3/issue/{key}".to_string(),
              format: "{fields.summary} (Issue {key})".to_string(),
            },
          ],
        }),
      ]),
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

  fn get(&self, domain: &Domain, url: &str) -> reqwest::RequestBuilder {
    let mut builder = self.client.get(url)
      .header("User-Agent", &format!("Unfurl/{}", VERSION));
    for (key, val) in &domain.headers {
      builder = builder.header(key, val);
    }
    domain.authenticate(builder)
  }
}

impl Service for Generic {
  fn request(&self, _conf: &config::Config, link: &url::Url) -> Result<reqwest::RequestBuilder, error::Error> {
    match self.find_route(link) {
      Some((domain, ept, mat)) => Ok(self.get(domain, &ept.url(link, &mat)?)),
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

