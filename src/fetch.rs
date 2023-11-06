use std::thread;
use std::sync::mpsc;

use futures::{stream, StreamExt};
use reqwest;
use once_cell::sync::OnceCell;

use crate::error;

static SERVICE: OnceCell<Service> = OnceCell::new();

struct Message {
  tx: mpsc::Sender<String>,
  urls: Vec<String>,
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

  pub fn send(&self, urls: Vec<String>) -> Result<mpsc::Receiver<String>, error::Error> {
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
        for rsp in fetch_n(3, x.urls).await {
          match rsp {
            Ok(rsp)  => println!(">>> COOL: {:?}", rsp.bytes().await),
            Err(err) => println!(">>> NAH BARF: {}", err),
          }
        }
        if let Err(err) = x.tx.send("Ok...".to_string()) {
          println!("*** Could not send: {}", err);
          return;
        }
      }
    }) 
  }
}

async fn fetch_n(n: usize, urls: Vec<String>) -> Vec<Result<reqwest::Response, error::Error>> {
  let client = reqwest::Client::new();
  
  let rsps = stream::iter(urls)
    .map(|url| {
      let client = &client;
      async move {
        Ok(client.get(url).send().await?)
      }
    })
    .buffer_unordered(n);

  rsps.collect().await
  // bodies
  //   .for_each(|b| async {
  //     match b {
  //       Ok(b) => println!("Got {} bytes", b.len()),
  //       Err(e) => eprintln!("Got an error: {}", e),
  //     }
  //   }).await;
}

