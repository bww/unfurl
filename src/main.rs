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
  let mut text: &str = &data;
  while text.len() > 0 {
    text = match text.find("https://") {
      Some(x) => unfurl_url(&text, x)?,
      None    => {
        print!("{}", text);
        &text[0..0]
      },
    };
  }
  Ok(())
}

fn unfurl_url<'a>(data: &'a str, x: usize) -> Result<&'a str, error::Error> {
  print!("{}", &data[..x]);
  let data: &str = &data[x..];
  let (url, rest) = match data.find(char::is_whitespace) {
    Some(y) => (&data[..y], &data[y..]),
    None    => (data, &data[0..0]),
  };

  let url = url::Url::parse(url)?;
  let host = match url.host_str() {
    Some(host) => host,
    None       => "<unknown>",
  };
  print!("<<{}>> ({})", url, host);

  Ok(rest)
}
