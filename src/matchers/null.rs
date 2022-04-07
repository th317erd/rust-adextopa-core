use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;

#[derive(Debug)]
pub struct NullPattern {}

impl NullPattern {
  pub fn new() -> MatcherRef {
    Rc::new(RefCell::new(Box::new(NullPattern {})))
  }
}

impl Matcher for NullPattern {
  fn exec(
    &self,
    _: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Skip(0))
  }

  fn is_consuming(&self) -> bool {
    false
  }

  fn get_name(&self) -> &str {
    "Null"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Null` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Null` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Null {
  () => {
    $crate::matchers::null::NullPattern::new()
  };
}

#[cfg(test)]
mod tests {
  use crate::{matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext};

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Null!();
    let result = ParserContext::tokenize(parser_context, matcher);

    assert_eq!(result, Ok(MatcherSuccess::Skip(0)));
  }
}
