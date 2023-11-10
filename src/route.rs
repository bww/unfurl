use std::path;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Match {
  pub vars: HashMap<String, String>,
}

impl Match {
  pub fn get<'a>(&'a self, name: &str) -> Option<&'a String> {
    self.vars.get(name)
  }
}

#[derive(Debug, PartialEq)]
pub struct Pattern (path::PathBuf);

impl Pattern {
  pub fn new<P: AsRef<path::Path>>(p: P) -> Pattern {
    Pattern(p.as_ref().to_path_buf())
  }

  pub fn match_path<P: AsRef<path::Path>>(&self, p: P) -> Option<Match> {
    let mut lit = self.0.components().into_iter();
    let mut rit = p.as_ref().components().into_iter();

    let mut vars: HashMap<String, String> = HashMap::new();
    loop {
      let lc = match lit.next(){
        Some(lc) => lc,
        None     => break,
      };
      let rc = match rit.next(){
        Some(rc) => rc,
        None     => break,
      };
      let lc = match lc {
        path::Component::RootDir   => if lc != rc { return None; } else { continue; },
        path::Component::CurDir    => if lc != rc { return None; } else { continue; },
        path::Component::ParentDir => if lc != rc { return None; } else { continue; },
        path::Component::Normal(v) => match v.to_str() {
          Some(v) => v,
          None    => return None,
        },
        _ => return None,
      };
      let rc = match rc {
        path::Component::Normal(v) => match v.to_str() {
          Some(v) => v,
          None    => return None,
        },
        _ => return None,
      };
      let ln = lc.len();
      if ln > 2 && &lc[0..1] == "{" && &lc[ln-1..ln] == "}" {
        vars.insert(lc[1..ln-1].to_string(), rc.to_string());
      }else if lc != rc {
        return None; // match failed
      }
    }
    
    if let Some(_) = lit.next() {
      return None; // not fully consumed
    };
    if let Some(_) = rit.next() {
      return None; // not fully consumed
    };

    Some(Match{
      vars: vars,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  impl Match {
    fn new(vars: HashMap<String, String>) -> Match {
      Match{
        vars: vars,
      }
    }

    fn new_empty() -> Match {
      Match{
        vars: HashMap::new(),
      }
    }
  }

  #[test]
  fn equality() {
    let a = Pattern::new("a/b");
    assert_eq!(a, a);
    let b = Pattern::new("a/b");
    assert_eq!(a, b);
  }

  #[test]
  fn match_path() {
    let p = Pattern::new("a/b");
    assert_eq!(None, p.match_path("a/c"));
    assert_eq!(None, p.match_path("/a/b"));
    assert_eq!(None, p.match_path("/a/b/"));

    let p = Pattern::new("/");
    assert_eq!(Some(Match::new_empty()), p.match_path("/"));
    let p = Pattern::new("a/b");
    assert_eq!(Some(Match::new_empty()), p.match_path("a/b"));
    let p = Pattern::new("/a/b");
    assert_eq!(Some(Match::new_empty()), p.match_path("/a/b"));
    let p = Pattern::new("a/{b}");
    assert_eq!(Some(Match::new(HashMap::from([("b".to_string(), "Hello".to_string())]))), p.match_path("a/Hello"));

    let p = Pattern::new("/{a}/{b}");
    assert_eq!(Some(Match::new(HashMap::from([
      ("a".to_string(), "Anything".to_string()),
      ("b".to_string(), "Hello".to_string()),
    ]))), p.match_path("/Anything/Hello"));
  }

}
