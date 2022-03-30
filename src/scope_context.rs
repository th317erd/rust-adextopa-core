use std::{cell::RefCell, rc::Rc};

use crate::scope::Scope;

struct ScopeContext<'a> {
  stack: Vec<Rc<RefCell<Scope<'a>>>>,
}

impl<'a> ScopeContext<'a> {
  pub fn new() -> Self {
    Self { stack: Vec::new() }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
