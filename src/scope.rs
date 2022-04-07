use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{matcher::MatcherRef, token::TokenRef};

#[derive(Debug, Clone)]
pub enum VariableType {
  Token(TokenRef),
  String(String),
  Matcher(MatcherRef),
}

pub type ScopeRef = Rc<RefCell<Scope>>;

#[derive(Debug, Clone)]
pub struct Scope {
  references: HashMap<String, VariableType>,
}

impl Scope {
  pub fn new() -> ScopeRef {
    Rc::new(RefCell::new(Self {
      references: HashMap::new(),
    }))
  }

  pub fn contains_key(&self, name: &str) -> bool {
    self.references.contains_key(name)
  }

  pub fn get(&self, name: &str) -> Option<&VariableType> {
    self.references.get(name)
  }

  pub fn set(&mut self, name: &str, value: VariableType) -> Option<VariableType> {
    self.references.insert(name.to_string(), value)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
