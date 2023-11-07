use crate::error;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token<'a> {
  Text(&'a str),
  URL(&'a str),
  EOF,
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
  match text.find(char::is_whitespace) {
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
    assert_eq!(Token::URL("https://google.com,"), tok);
    assert_eq!(" and then trailing.", text);
    let (tok, text) = next(text);
    assert_eq!(Token::Text(" and then trailing."), tok);
    assert_eq!("", text);
    let (tok, text) = next(text);
    assert_eq!(Token::EOF, tok);
    assert_eq!("", text);
  }

}

