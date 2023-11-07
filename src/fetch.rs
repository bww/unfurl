use std::thread;
use std::sync::mpsc;

use bytes::Bytes;
use futures::{stream, StreamExt};
use reqwest;
use once_cell::sync::OnceCell;

use crate::error;

const CONCURRENT_REQUESTS: usize  = 3;

static SERVICE: OnceCell<Service> = OnceCell::new();

#[derive(Debug)]
struct Request {
  key: String,
  req: reqwest::RequestBuilder,
}

#[derive(Debug)]
struct Requests {
  tx: mpsc::Sender<Vec<Result<Response, error::Error>>>,
  reqs: Vec<Request>,
}

#[derive(Debug)]
pub struct Response {
  key: String,
  data: Bytes,
}

impl Response {
  pub fn key<'a>(&'a self) -> &'a str {
    &self.key
  }

  pub fn data<'a>(&'a self) -> &'a Bytes {
    &self.data
  }
}

pub struct Service {
  client: reqwest::Client,
  tx: mpsc::Sender<Requests>,
}

impl Service {
  pub fn instance() -> &'static Service {
    SERVICE.get_or_init(|| { Self::new() })
  }

  fn new() -> Service {
    let (q_tx, q_rx) = mpsc::channel();
    let svc = Service{
      client: reqwest::Client::new(),
      tx: q_tx,
    };
    thread::spawn(|| { Service::run(reqwest::Client::new(), q_rx) });
    svc
  }

  pub fn fetch_urls(&self, urls: Vec<String>) -> Result<mpsc::Receiver<Vec<Result<Response, error::Error>>>, error::Error> {
    self.fetch_requests(urls.iter().map(|e| {
      Request{
        key: e.to_string(),
        req: self.client.get(e),
      }
    }).collect())
  }

  pub fn fetch_requests(&self, reqs: Vec<Request>) -> Result<mpsc::Receiver<Vec<Result<Response, error::Error>>>, error::Error> {
    let (p_tx, p_rx) = mpsc::channel();
    match self.tx.send(Requests{tx: p_tx, reqs: reqs}) {
      Ok(_)    => Ok(p_rx),
      Err(err) => Err(error::Error::SendError),
    }
  }

  fn run(client: reqwest::Client, rx: mpsc::Receiver<Requests>) {
    let client = &client;
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
      loop {
        let x = match rx.recv() {
          Ok(x)    => x,
          Err(err) => {
            println!("* * * Could not receive: {}", err);
            return;
          },
        };
        let rsps = fetch_n(client, CONCURRENT_REQUESTS, x.reqs).await;
        if let Err(err) = x.tx.send(rsps) {
          println!("*** Could not send: {}", err);
          return;
        }
      }
    }) 
  }
}

async fn fetch_n(client: &reqwest::Client, n: usize, reqs: Vec<Request>) -> Vec<Result<Response, error::Error>> {
  stream::iter(reqs)
    .map(|req| {
      let client = &client;
      async move {
        Ok(Response{
          key: req.key.clone(),
          data: req.req.send().await?.bytes().await?,
        })
      }
    })
    .buffer_unordered(n)
    .collect()
    .await
}

