use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct OptionalPattern {
  matcher: MatcherRef,
}

impl OptionalPattern {
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
      Ok(success) => Ok(success),
      Err(failure) => match failure {
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(message, range) => Err(MatcherFailure::Error(message, range)),
      },
    }
  }
}

impl Matcher for OptionalPattern {
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
    "Optional"
  }

  fn set_name(&mut self, name: &str) {
    // panic!("Can not set `name` on a `Optional` matcher");
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
    panic!("Can not add a pattern to a `Optional` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Optional {
  ($arg:expr) => {
    $crate::matchers::optional::OptionalPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Optional!(Equals!("Testing"));

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Optional!(Equals!("testing"));

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
