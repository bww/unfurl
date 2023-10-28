use std::io::{Read};
use std::fs;
use std::env;

mod error;

fn main() {
  match app() {
    Ok(_)    => {},
    Err(err) => eprintln!("* * * {}", err),
  }
}

fn app() -> Result<(), error::Error> {
  let input = env::args().nth(1);
  match input {
    Some(path) => unfurl(fs::File::open(path)?),
    None       => unfurl(std::io::stdin()),
  }
}

fn unfurl<R: Read>(mut r: R) -> Result<(), error::Error> {
  let mut data = String::new();
  r.read_to_string(&mut data)?;
  println!(">>> YO:\n{}", data);
  Ok(())
}

