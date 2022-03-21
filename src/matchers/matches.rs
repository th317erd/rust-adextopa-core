extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::token::StandardToken;
use regex::Regex;

pub struct EqualsPattern<'a> {
  pattern: &'a str,
  name: &'a str,
}

impl<'a> EqualsPattern<'a> {
  pub fn new(pattern: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      pattern,
      name: "Equals",
    })))
  }

  pub fn new_with_name(name: &'a str, pattern: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { pattern, name })))
  }
}

impl<'a> Matcher<'a> for EqualsPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    if let Some(range) = context.borrow().matches_str(self.pattern) {
      Ok(MatcherSuccess::Token(StandardToken::new(
        &context.borrow().parser,
        self.name.to_string(),
        range,
      )))
    } else {
      Err(MatcherFailure::Fail)
    }
  }

  fn get_name(&self) -> &str {
    self.name
  }

  fn set_name(&mut self, name: &'a str) {
    self.name = name;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a Equals pattern");
  }
}

pub struct MatchesPattern<'a> {
  regex: Regex,
  name: &'a str,
}

impl<'a> MatchesPattern<'a> {
  pub fn new(regex: Regex) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      regex,
      name: "Matches",
    })))
  }

  pub fn new_with_name(name: &'a str, regex: Regex) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { regex, name })))
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

  fn get_name(&self) -> &str {
    self.name
  }

  fn set_name(&mut self, name: &'a str) {
    self.name = name;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a Matches pattern");
  }
}

#[macro_export]
macro_rules! Equals {
  ($name:expr; $arg:expr) => {
    $crate::matchers::matches::EqualsPattern::new_with_name($name, $arg)
  };

  ($arg:expr) => {
    $crate::matchers::matches::EqualsPattern::new($arg)
  };
}

#[macro_export]
macro_rules! Matches {
  ($name:expr; $arg:expr) => {
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
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Equals!("Testing");

    if let Ok(MatcherSuccess::Token(token)) = matcher.borrow().exec(parser_context.clone()) {
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
      matcher.borrow().exec(parser_context.clone()),
      Err(MatcherFailure::Fail)
    );
  }

  #[test]
  fn it_matches_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r"\w+");

    if let Ok(MatcherSuccess::Token(token)) = matcher.borrow().exec(parser_context.clone()) {
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

    if let Ok(MatcherSuccess::Token(token)) = matcher.borrow().exec(parser_context.clone()) {
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
      matcher.borrow().exec(parser_context.clone()),
      Err(MatcherFailure::Fail)
    );
  }
}
