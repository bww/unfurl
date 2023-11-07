use std::thread;
use std::sync::mpsc;

use bytes::Bytes;
use futures::{stream, StreamExt};
use reqwest;
use once_cell::sync::OnceCell;

use crate::error;

const CONCURRENT_REQUESTS: usize  = 3;

static SERVICE: OnceCell<Service> = OnceCell::new();

struct Message {
  tx: mpsc::Sender<Vec<Result<Response, error::Error>>>,
  urls: Vec<String>,
}

#[derive(Debug)]
pub struct Response {
  url: String,
  data: Bytes,
}

impl Response {
  pub fn url<'a>(&'a self) -> &'a str {
    &self.url
  }

  pub fn data<'a>(&'a self) -> &'a Bytes {
    &self.data
  }
}

pub struct Service {
  tx: mpsc::Sender<Message>,
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

  pub fn send(&self, urls: Vec<String>) -> Result<mpsc::Receiver<Vec<Result<Response, error::Error>>>, error::Error> {
    let (p_tx, p_rx) = mpsc::channel();
    match self.tx.send(Message{tx: p_tx, urls: urls}) {
      Ok(_)    => Ok(p_rx),
      Err(err) => Err(error::Error::SendError),
    }
  }

  fn run(rx: mpsc::Receiver<Message>) {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
      loop {
        let x = match rx.recv() {
          Ok(x)    => x,
          Err(err) => {
            println!("* * * Could not receive: {}", err);
            return;
          },
        };
        let rsps = fetch_n(CONCURRENT_REQUESTS, x.urls).await;
        if let Err(err) = x.tx.send(rsps) {
          println!("*** Could not send: {}", err);
          return;
        }
      }
    }) 
  }
}

async fn fetch_n(n: usize, urls: Vec<String>) -> Vec<Result<Response, error::Error>> {
  let client = reqwest::Client::new();
  
  let rsps = stream::iter(urls)
    .map(|url| {
      let client = &client;
      async move {
        Ok(Response{
          url: url.clone(),
          data: client.get(url).send().await?.bytes().await?,
        })
      }
    })
    .buffer_unordered(n);

  rsps.collect().await
}

