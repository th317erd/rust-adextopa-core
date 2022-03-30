use std::collections::HashMap;

use crate::{matcher::MatcherRef, token::TokenRef};

pub enum VariableType<'a> {
  Token(TokenRef),
  String(String),
  Matcher(MatcherRef<'a>),
}

pub struct Scope<'a> {
  references: HashMap<String, VariableType<'a>>,
}

impl<'a> Scope<'a> {
  pub fn new() -> Self {
    Self {
      references: HashMap::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
