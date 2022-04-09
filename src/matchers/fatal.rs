use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;

#[derive(Debug)]
pub struct FatalPattern {
  message: String,
}

impl FatalPattern {
  pub fn new(message: &str) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      message: message.to_string(),
    })))
  }

  fn _exec(
    &self,
    _: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    Err(MatcherFailure::Error(self.message.to_string()))
  }
}

impl Matcher for FatalPattern {
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
    "Error"
  }

  fn set_name(&mut self, _: &str) {
    // panic!("Can not set `name` on a `Fatal` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Fatal` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Fatal {
  ($message:expr) => {
    $crate::matchers::fatal::FatalPattern::new($message)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext, Discard, Matches,
    Program,
  };

  #[test]
  fn it_can_throw_a_fatal_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Matches!(r"\w+"),
      Fatal!("There was an error!"),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+")
    );

    if let Err(MatcherFailure::Error(message)) = ParserContext::tokenize(parser_context, matcher) {
      assert_eq!(message, "There was an error!");
    } else {
      unreachable!("Test failed!");
    };
  }
}
