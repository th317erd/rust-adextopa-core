extern crate adextopa_macros;
use core::panic;

use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess, Pattern};
use crate::parser_context::ParserContextRef;
use crate::token::StandardToken;

pub struct MatchesPattern<'a> {
  pattern: Pattern<'a>,
  name: &'a str,
}

impl<'a> MatchesPattern<'a> {
  pub fn new(pattern: Pattern<'a>) -> Self {
    let name = match pattern {
      Pattern::String(_) => "Equals",
      Pattern::RegExp(_) => "Matches",
      _ => panic!("MatchesPattern: token can only match against strings and regular expressions"),
    };

    Self { pattern, name }
  }

  pub fn new_with_name(name: &'a str, pattern: Pattern<'a>) -> MatchesPattern<'a> {
    Self { pattern, name }
  }

  pub fn set_name(&mut self, name: &'a str) {
    self.name = name;
  }
}

impl<'a> Matcher for MatchesPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match self.pattern {
      Pattern::String(s) => {
        // println!("Attempting to EQUAL against: {:?} [{}]", s, context.borrow().debug_range(10));
        if let Some(range) = context.borrow().matches_str(s) {
          Ok(MatcherSuccess::Token(StandardToken::new(&context.borrow().parser, self.name, range)))
        } else {
          Err(MatcherFailure::Fail)
        }
      }
      Pattern::RegExp(ref re) => {
        // println!("Attempting to MATCH against: {:?} [{}]", re, context.borrow().debug_range(10));

        if let Some(range) = context.borrow().matches_regexp(re) {
          // We got a match, but it has zero length
          // In this case, respond with a "Skip"
          if range.start == range.end {
            return Ok(MatcherSuccess::Skip(0));
          }

          Ok(MatcherSuccess::Token(StandardToken::new(&context.borrow().parser, self.name, range)))
        } else {
          Err(MatcherFailure::Fail)
        }
      }
      _ => unreachable!("MatchesPattern: attempting to match against something that isn't a string or a regular expression"),
    }
  }

  fn get_name(&self) -> &str {
    self.name
  }
}

#[macro_export]
macro_rules! Equals {
  ($name:expr; $arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new_with_name(
      $name,
      $crate::matcher::Pattern::String($arg),
    )
  };

  ($arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new($crate::matcher::Pattern::String($arg))
  };
}

#[macro_export]
macro_rules! Matches {
  ($name:expr; $arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new_with_name(
      $name,
      $crate::matcher::Pattern::RegExp(regex::Regex::new($arg).unwrap()),
    )
  };

  ($arg:expr) => {
    $crate::matchers::matches::MatchesPattern::new($crate::matcher::Pattern::RegExp(
      regex::Regex::new($arg).unwrap(),
    ))
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Equals!("Testing");

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
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
      matcher.exec(parser_context.clone()),
      Err(MatcherFailure::Fail)
    );
  }

  #[test]
  fn it_matches_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r"\w+");

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
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

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
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
      matcher.exec(parser_context.clone()),
      Err(MatcherFailure::Fail)
    );
  }
}
