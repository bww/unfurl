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
    let (w_tx, w_rx) = mpsc::channel();
    let svc = Service{tx: w_tx};
    thread::spawn(|| { Service::run(w_rx) });
    svc
  }

  pub fn send(&self, urls: Vec<String>) -> Result<mpsc::Receiver<String>, error::Error> {
    let (r_tx, r_rx) = mpsc::channel();
    match self.tx.send(Message{tx: r_tx, urls: urls}) {
      Ok(_)    => Ok(r_rx),
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
        fetch_n(3, x.urls);
        // for u in x.urls {
        //   if let Err(err) = x.tx.send(x.data) {
        //     println!("*** Could not send: {}", err);
        //     return;
        //   }
        // }
      }
    }) 
  }
}

async fn fetch_n(n: usize, urls: Vec<String>) {
  let client = reqwest::Client::new();
  
  let bodies = stream::iter(urls)
    .map(|url| {
      let client = &client;
      async move {
        let resp = client.get(url).send().await?;
        resp.bytes().await
      }
    })
    .buffer_unordered(n);

  bodies
    .for_each(|b| async {
      match b {
        Ok(b) => println!("Got {} bytes", b.len()),
        Err(e) => eprintln!("Got an error: {}", e),
      }
    }).await;
}

