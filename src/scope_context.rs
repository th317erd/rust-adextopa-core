use std::{cell::RefCell, rc::Rc};

use crate::scope::{Scope, ScopeRef, VariableType};

pub type ScopeContextRef = Rc<RefCell<ScopeContext>>;

#[derive(Debug, Clone)]
pub struct ScopeContext {
  stack: Vec<ScopeRef>,
}

impl ScopeContext {
  pub fn new() -> ScopeContextRef {
    Rc::new(RefCell::new(Self { stack: Vec::new() }))
  }

  pub fn push(&mut self, scope: ScopeRef) {
    self.stack.push(scope);
  }

  pub fn pop(&mut self) -> Option<ScopeRef> {
    self.stack.pop()
  }

  pub fn get(&self, name: &str) -> Option<VariableType> {
    if self.stack.len() == 0 {
      return None;
    }

    for scope in (&self.stack).into_iter().rev() {
      match scope.borrow().get(name) {
        Some(value) => match value {
          VariableType::Token(token) => return Some(VariableType::Token(token.clone())),
          VariableType::String(value) => return Some(VariableType::String(value.clone())),
          VariableType::Matcher(matcher) => return Some(VariableType::Matcher(matcher.clone())),
        },
        None => continue,
      }
    }

    None
  }

  pub fn set(&mut self, name: &str, value: VariableType) -> Option<VariableType> {
    if self.stack.len() == 0 {
      self.stack.push(Scope::new());
    }

    let top_index = self.stack.len() - 1;
    let top = &mut self.stack[top_index];
    top.borrow_mut().set(name, value)
  }
}

#[cfg(test)]
mod tests {
  use crate::scope::{Scope, VariableType};

  use super::ScopeContext;

  #[test]
  fn it_works() {
    let scope_context = ScopeContext::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();

    scope1
      .borrow_mut()
      .set("Test", VariableType::String("Hello World".to_string()));

    scope2
      .borrow_mut()
      .set("Test", VariableType::String("Hello World 2".to_string()));

    scope_context.borrow_mut().push(scope1);
    scope_context.borrow_mut().push(scope2);

    if let Some(result) = scope_context.borrow().get("Test") {
      match result {
        VariableType::String(value) => assert_eq!(value, "Hello World 2"),
        _ => unreachable!("Test failed!"),
      }
    }

    scope_context.borrow_mut().pop();

    let _sc = scope_context.borrow();
    if let Some(result) = _sc.get("Test") {
      match result {
        VariableType::String(value) => assert_eq!(value, "Hello World"),
        _ => unreachable!("Test failed!"),
      }
    }
  }
}
