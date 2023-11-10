use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'a> {
  Text(&'a str),
  URL(&'a str),
  EOF,
}

impl<'a> fmt::Display for Token<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Text(text) => write!(f, "{}", text),
      Self::URL(url) => write!(f, "<{}>", url),
      Self::EOF => write!(f, "%"),
    }
  }
}

pub fn next<'a>(text: &'a str) -> (Token<'a>, &'a str) {
  if text.len() == 0 {
    return (Token::EOF, "");
  }
  let x = match text.find("https://") {
    Some(x) => x,
    None    => return (Token::Text(text), ""),
  };
  if x > 0 {
    return (Token::Text(&text[..x]), &text[x..]);
  }
  match next_url_end(text) {
    Some(y) => (Token::URL(&text[..y]), &text[y..]),
    None    => (Token::URL(text), ""),
  }
}

fn is_maybe_url(c: char) -> bool {
  return c == '.' || c == ':'
}

fn next_url_end(text: &str) -> Option<usize> {
  let text = text.chars();
  let mut p: Option<char> = None;
  for (i, c) in text.enumerate() {
    if char::is_whitespace(c) {
      match p {
        Some(p) => if is_maybe_url(p) {
          return Some(i - 1); // move back one
        } else {
          return Some(i);
        },
        None => return Some(i),
      }
    }
    if c == ',' || c == ';' || c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}' {
      return Some(i);
    }
    p = Some(c);
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn parse_text() {
    let text = "Hello, there: https://google.com, and then trailing. Also https://yahoo.com.";
    let (tok, text) = next(text);
    assert_eq!(Token::Text("Hello, there: "), tok);
    assert_eq!("https://google.com, and then trailing. Also https://yahoo.com.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::URL("https://google.com"), tok);
    assert_eq!(", and then trailing. Also https://yahoo.com.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::Text(", and then trailing. Also "), tok);
    assert_eq!("https://yahoo.com.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::URL("https://yahoo.com"), tok);
    assert_eq!(".", text);
    let (tok, text) = next(text);
    assert_eq!(Token::Text("."), tok);
    assert_eq!("", text);
    let (tok, text) = next(text);
    assert_eq!(Token::EOF, tok);
    assert_eq!("", text);
  }

}

