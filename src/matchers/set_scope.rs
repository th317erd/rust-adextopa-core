use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope::Scope;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct SetScopePattern {
  scope: Option<ScopeContextRef>,
  matcher: MatcherRef,
}

impl SetScopePattern {
  pub fn new(matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      scope: None,
      matcher,
    })))
  }

  pub fn new_with_scope(scope: ScopeContextRef, matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      scope: Some(scope.clone()),
      matcher,
    })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let scope = match &self.scope {
      Some(scope) => scope.clone(),
      None => {
        // Push a new scope onto the stack
        let scope = Scope::new();
        context.borrow().scope.borrow_mut().push(scope.clone());

        context.borrow().scope.clone()
      }
    };

    let result = self.matcher.borrow().exec(
      self.matcher.clone(),
      context.borrow().clone_with_name(self.get_name()),
      scope,
    );

    if self.scope.is_none() {
      // Pop scope from stack
      context.borrow().scope.borrow_mut().pop();
    }

    result
  }
}

impl Matcher for SetScopePattern {
  fn exec(
    &self,
    this_matcher: MatcherRef,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    self.before_exec(this_matcher.clone(), context.clone(), scope.clone());
    let result = self._exec(context.clone(), scope.clone());
    self.after_exec(this_matcher.clone(), context.clone(), scope.clone());

    result
  }

  fn get_name(&self) -> &str {
    "SetScope"
  }

  fn set_name(&mut self, name: &str) {
    // panic!("Can not set `name` on a `SetScope` matcher");
    self.matcher.borrow_mut().set_name(name);
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef) {
    if index > 0 {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.matcher = matcher;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    Some(vec![self.matcher.clone()])
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `SetScope` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! SetScope {
  ($scope:expr, $arg:expr) => {
    $crate::matchers::set_scope::SetScopePattern::new_with_scope($scope, $arg)
  };

  ($arg:expr) => {
    $crate::matchers::set_scope::SetScopePattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    parser::Parser, parser_context::ParserContext, scope::VariableType,
    scope_context::ScopeContext, source_range::SourceRange, Equals, Store,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let new_scope = ScopeContext::new();
    let matcher = SetScope!(new_scope.clone(), Store!("StoredValue"; Equals!("Testing")));

    if let Ok(token) = ParserContext::tokenize(parser_context.clone(), matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");

      if let Some(VariableType::Token(token)) = new_scope.borrow().get("StoredValue") {
        let token = token.borrow();
        assert_eq!(token.get_name(), "Equals");
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
        assert_eq!(token.get_value(), "Testing");
      } else {
        unreachable!("Test failed!");
      }

      assert_eq!(
        parser_context
          .borrow()
          .get_scope_variable("StoredValue")
          .is_none(),
        true
      );

      match new_scope.borrow().get("StoredValue") {
        Some(VariableType::Token(token)) => {
          assert_eq!(token.borrow().get_name(), "Equals")
        }
        _ => unreachable!("Test failed!"),
      }
    } else {
      unreachable!("Test failed!");
    };
  }
}
