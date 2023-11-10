use std::thread;
use std::sync::mpsc;

use bytes::Bytes;
use futures::{stream, StreamExt};
use once_cell::sync::OnceCell;
use reqwest;

use crate::error;

const CONCURRENT_REQUESTS: usize = 5;

static SERVICE: OnceCell<Service> = OnceCell::new();

#[derive(Debug)]
pub struct Request {
  key: String,
  req: reqwest::RequestBuilder,
}

impl Request{
  pub fn new(key: &str, req: reqwest::RequestBuilder) -> Self {
    Request{
      key: key.to_string(),
      req: req,
    }
  }
}

#[derive(Debug)]
struct Requests {
  tx: mpsc::Sender<Vec<Response>>,
  reqs: Vec<Request>,
}

#[derive(Debug)]
pub struct Response {
  key: String,
  data: Result<Bytes, error::Error>,
}

impl Response {
  pub fn key<'a>(&'a self) -> &'a str {
    &self.key
  }

  pub fn data<'a>(&'a self) -> &'a Result<Bytes, error::Error> {
    &self.data
  }
}

pub struct Service {
  tx: mpsc::Sender<Requests>,
}

impl Service {
  pub fn instance() -> &'static Service {
    SERVICE.get_or_init(|| { Self::new() })
  }

  fn new() -> Service {
    let (q_tx, q_rx) = mpsc::channel();
    let svc = Service{tx: q_tx};
    thread::spawn(|| { Service::run(q_rx) });
    svc
  }

  pub fn fetch_requests(&self, reqs: Vec<Request>) -> Result<mpsc::Receiver<Vec<Response>>, error::Error> {
    let (p_tx, p_rx) = mpsc::channel();
    match self.tx.send(Requests{tx: p_tx, reqs: reqs}) {
      Ok(_)  => Ok(p_rx),
      Err(_) => Err(error::Error::SendError),
    }
  }

  fn run(rx: mpsc::Receiver<Requests>) {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
      loop {
        let x = match rx.recv() {
          Ok(x)    => x,
          Err(err) => {
            println!("* * * Could not receive: {}", err);
            return;
          },
        };
        let rsps = fetch_n(CONCURRENT_REQUESTS, x.reqs).await;
        if let Err(err) = x.tx.send(rsps) {
          println!("*** Could not send: {}", err);
          return;
        }
      }
    }) 
  }
}

async fn fetch_n(n: usize, reqs: Vec<Request>) -> Vec<Response> {
  stream::iter(reqs)
    .map(|req| {
      async move {
        Response{
          key: req.key.clone(),
          data: match req.req.send().await {
            Err(err) => Err(err.into()),
            Ok(rsp)  => match rsp.error_for_status() {
              Ok(rsp) => match rsp.bytes().await {
                Ok(data) => Ok(data),
                Err(err) => Err(err.into()),
              },
              Err(err) => Err(err.into()),
            },
          }
        }
      }
    })
    .buffer_unordered(n)
    .collect()
    .await
}

