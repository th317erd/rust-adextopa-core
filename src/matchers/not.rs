use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;

#[derive(Debug)]
pub struct NotPattern {
  matcher: MatcherRef,
}

impl NotPattern {
  pub fn new(matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    match self.matcher.borrow().exec(
      self.matcher.clone(),
      context.borrow().clone_with_name(self.get_name()),
      scope.clone(),
    ) {
      Ok(success) => match success {
        // Fail on success
        MatcherSuccess::Token(_) => return Err(MatcherFailure::Fail),
        MatcherSuccess::ProxyChildren(_) => return Err(MatcherFailure::Fail),
        MatcherSuccess::Skip(amount) => {
          // If Skip value is anything but zero, then fail
          if amount != 0 {
            return Err(MatcherFailure::Fail);
          }

          return Ok(success);
        }
        // For other success types (Skip, Stop, Break, Continue, None) succeed
        _ => return Ok(success),
      },
      Err(failure) => match failure {
        // Succeed on fail
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(error) => Err(MatcherFailure::Error(error)),
      },
    }
  }
}

impl Matcher for NotPattern {
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
    "Not"
  }

  fn set_name(&mut self, name: &str) {
    // panic!("Can not set `name` on a `Not` matcher");
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
    panic!("Can not add a pattern to a Not pattern");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Not {
  ($arg:expr) => {
    $crate::matchers::not::NotPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Not!(Equals!("Testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Not!(Equals!("testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      matcher.borrow().exec(
        matcher.clone(),
        parser_context.clone(),
        parser_context.borrow().scope.clone(),
      )
    );
  }
}
