use std::thread;
use std::sync::mpsc;

// use reqwest;
use once_cell::sync::OnceCell;

use crate::error;

static SERVICE: OnceCell<Service> = OnceCell::new();

pub struct Service {
  rx: mpsc::Receiver<String>,
  tx: mpsc::Sender<String>,
}

impl Service {
  pub fn instance() -> &'static Service {
    SERVICE.get_or_init(|| {
      Self::new()
    })
  }

  fn new() -> Service {
    let (w_tx, w_rx) = mpsc::channel();
    let (r_tx, r_rx) = mpsc::channel();
    let svc = Service{
      rx: r_rx,
      tx: w_tx,
    };
    thread::spawn(|| { Service::run(w_rx, r_tx) });
    svc
  }

  pub fn send(&self, elem: String) -> Result<(), error::Error> {
    match self.tx.send(elem) {
      Ok(_)    => Ok(()),
      Err(err) => Err(error::Error::SendError),
    }
  }

  pub fn recv(&self) -> Result<String, error::Error> {
    self.rx.recv()
  }

  fn run(rx: mpsc::Receiver<String>, tx: mpsc::Sender<String>) {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
      loop {
        let x = match rx.recv() {
          Ok(x)    => x,
          Err(err) => {
            println!("*** Could not receive: {}", err);
            return;
          },
        };
        match tx.send(x) {
          Ok(_)    => Ok(()),
          Err(err) => {
            println!("*** Could not send: {}", err);
            return;
          },
        };
      }
    }) 
  }
}

// pub fn fetch_n() {
//   let rt = match runtime::Handle::current() {
//     Some(rt) => rt,
//     None     => runtime::Runtime::new()?,
//   };
// }
// 
// fn fetch_n_exec() {
//   let bodies = stream::iter(urls)
//     .map(|url| {
//       let client = &client;
//       async move {
//         let resp = client.get(url).send().await?;
//         resp.bytes().await
//       }
//     })
//     .buffer_unordered(CONCURRENT_REQUESTS);
// 
//   bodies
//     .for_each(|b| async {
//       match b {
//         Ok(b) => println!("Got {} bytes", b.len()),
//         Err(e) => eprintln!("Got an error: {}", e),
//       }
//     }).await;
// }
// 
