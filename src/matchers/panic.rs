use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parse_error::ParseError;
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::source_range::SourceRange;

#[derive(Debug)]
pub struct PanicPattern {
  message: String,
}

impl PanicPattern {
  pub fn new(message: &str) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      message: message.to_string(),
    })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let range = match context.borrow().get_top_token_from_stack() {
      Some(token) => {
        let start = token.borrow().get_matched_range().start;
        SourceRange::new(start, context.borrow().offset.start)
      }
      None => {
        let start = context.borrow().offset.start;
        SourceRange::new(start, start)
      }
    };

    Err(MatcherFailure::Error(ParseError::new_with_range(
      &context.borrow().get_error_as_string(&self.message, &range),
      range,
    )))
  }
}

impl Matcher for PanicPattern {
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
    // panic!("Can not set `name` on a `Panic` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Panic` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Panic {
  ($message:expr) => {
    $crate::matchers::panic::PanicPattern::new($message)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Discard, Matches, Program,
  };

  #[test]
  fn it_can_throw_a_fatal_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Matches!(r"\w+"),
      Panic!("There was an error!"),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+")
    );

    if let Err(MatcherFailure::Error(error)) = ParserContext::tokenize(parser_context, matcher) {
      assert_eq!(error.message, "Error: @[1:1-8]: There was an error!");
      assert_eq!(error.range, Some(SourceRange::new(0, 7)));
    } else {
      unreachable!("Test failed!");
    };
  }
}
