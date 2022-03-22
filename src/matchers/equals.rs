extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::token::StandardToken;

use super::fetch::Fetchable;

pub struct EqualsPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  pattern: T,
  name: &'a str,
  custom_name: bool,
}

impl<'a, T> EqualsPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  pub fn new(pattern: T) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      pattern,
      name: "Equals",
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &'a str, pattern: T) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      pattern,
      name,
      custom_name: true,
    })))
  }
}

impl<'a, T> Matcher<'a> for EqualsPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let pattern_value = self.pattern.fetch_value(sub_context);
    if let Some(range) = context.borrow().matches_str(pattern_value.as_str()) {
      Ok(MatcherSuccess::Token(StandardToken::new(
        &context.borrow().parser,
        self.name.to_string(),
        range,
      )))
    } else {
      Err(MatcherFailure::Fail)
    }
  }

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name
  }

  fn set_name(&mut self, name: &'a str) {
    self.name = name;
    self.custom_name = true;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Equals` matcher");
  }
}

#[macro_export]
macro_rules! Equals {
  ($name:literal; $arg:expr) => {
    $crate::matchers::equals::EqualsPattern::new_with_name($name, $arg)
  };

  ($arg:expr) => {
    $crate::matchers::equals::EqualsPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Equals!("Testing");

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Equals!("testing");

    assert_eq!(
      ParserContext::tokenize(parser_context, matcher),
      Err(MatcherFailure::Fail)
    );
  }
}
