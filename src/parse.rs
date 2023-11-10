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
  match text.find(|c| {
    char::is_whitespace(c) || c == ',' || c == ';' || c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}'
  }) {
    Some(y) => (Token::URL(&text[..y]), &text[y..]),
    None    => (Token::URL(text), ""),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn parse_text() {
    let text = "Hello, there: https://google.com, and then trailing.";
    let (tok, text) = next(text);
    assert_eq!(Token::Text("Hello, there: "), tok);
    assert_eq!("https://google.com, and then trailing.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::URL("https://google.com"), tok);
    assert_eq!(", and then trailing.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::Text(", and then trailing."), tok);
    assert_eq!("", text);
    let (tok, text) = next(text);
    assert_eq!(Token::EOF, tok);
    assert_eq!("", text);
  }

}

