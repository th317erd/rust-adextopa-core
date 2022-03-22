extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::token::StandardToken;
use regex::Regex;

pub struct MatchesPattern<'a> {
  regex: Regex,
  name: &'a str,
  custom_name: bool,
}

impl<'a> MatchesPattern<'a> {
  pub fn new(regex: Regex) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      regex,
      name: "Matches",
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &'a str, regex: Regex) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      regex,
      name,
      custom_name: true,
    })))
  }
}

impl<'a> Matcher<'a> for MatchesPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    if let Some(range) = context.borrow().matches_regexp(&self.regex) {
      // We got a match, but it has zero length
      // In this case, respond with a "Skip"
      if range.start == range.end {
        return Ok(MatcherSuccess::Skip(0));
      }

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
    panic!("Can not add a pattern to a `Matches` matcher");
  }
}

#[macro_export]
macro_rules! Matches {
  ($name:literal; $arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new_with_name(
      $name,
      regex::Regex::new($arg).unwrap(),
    )
  };

  ($arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new(regex::Regex::new($arg).unwrap())
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
  fn it_matches_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r"\w+");

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Matches");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_regexp_with_a_non_zero_offset() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r".+");

    parser_context.borrow_mut().offset.start = 8;

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Matches");
      assert_eq!(*token.get_value_range(), SourceRange::new(8, 12));
      assert_eq!(token.value(), "1234");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r"\d+");

    let t = Box::<i32>::new(20);
    Box::leak(t);

    assert_eq!(
      ParserContext::tokenize(parser_context, matcher),
      Err(MatcherFailure::Fail)
    );
  }
}
