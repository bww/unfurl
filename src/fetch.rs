use std::thread;
use std::sync::mpsc;

// use reqwest;
use once_cell::sync::OnceCell;

use crate::error;

static SERVICE: OnceCell<Service> = OnceCell::new();

pub struct Service {
  tx: mpsc::Sender<String>,
}

impl Service {
  pub fn instance() -> &'static Service {
    SERVICE.get_or_init(|| {
      Self::new()
    })
  }

  fn new() -> Service {
    let (tx, rx) = mpsc::channel();
    let svc = Service{
      tx: tx,
    };
    thread::spawn(|| { Service::run(rx) });
    svc
  }

  pub fn send(&self, elem: String) -> Result<(), error::Error> {
    match self.tx.send(elem) {
      Ok(_)    => Ok(()),
      Err(err) => Err(error::Error::SendError),
    }
  }

  fn run(rx: mpsc::Receiver<String>) {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
      loop {
        let x = match rx.recv() {
          Ok(x)    => x,
          Err(err) => {
            println!("*** Could not receive: {}", err);
            return;
          },
        };
        println!("Hello world {}", x);
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
