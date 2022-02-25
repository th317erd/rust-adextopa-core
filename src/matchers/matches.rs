use core::panic;

use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess, Pattern};
use crate::parser_context::ParserContext;
use crate::token::Token;

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

  pub fn new_with_name(pattern: Pattern<'a>, name: &'a str) -> MatchesPattern<'a> {
    Self { pattern, name }
  }
}

impl<'a> Matcher for MatchesPattern<'a> {
  fn exec(&self, context: &ParserContext) -> Result<MatcherSuccess, MatcherFailure> {
    match self.pattern {
      Pattern::String(s) => {
        if let Some(range) = context.matches_str(s) {
          Ok(MatcherSuccess::Token(Token::new(self.name, range)))
        } else {
          Err(MatcherFailure::Fail)
        }
      }
      Pattern::RegExp(ref re) => {
        if let Some(range) = context.matches_regexp(re) {
          Ok(MatcherSuccess::Token(Token::new(self.name, range)))
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
  ($arg:expr) => {
    crate::matchers::matches::MatchesPattern::new(crate::matcher::Pattern::String($arg))
  };
}

#[macro_export]
macro_rules! Matches {
  ($arg:expr) => {{
    crate::matchers::matches::MatchesPattern::new(crate::matcher::Pattern::RegExp(
      regex::Regex::new($arg).unwrap(),
    ))
  }};
}

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
    let parser_context = ParserContext::new(&parser);
    let matcher = Equals!("Testing");

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Equals");
      assert_eq!(token.value_range, SourceRange::new(0, 7));
      assert_eq!(token.value(&parser), "Testing");
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Equals!("testing");

    assert_eq!(matcher.exec(&parser_context), Err(MatcherFailure::Fail));
  }

  #[test]
  fn it_matches_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Matches!(r"\w+");

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Matches");
      assert_eq!(token.value_range, SourceRange::new(0, 7));
      assert_eq!(token.value(&parser), "Testing");
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_fails_to_match_against_a_regexp_with_a_non_zero_offset() {
    let parser = Parser::new("Testing 1234");
    let mut parser_context = ParserContext::new(&parser);
    let matcher = Matches!(r".+");

    parser_context.offset.start = 8;

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Matches");
      assert_eq!(token.value_range, SourceRange::new(8, 12));
      assert_eq!(token.value(&parser), "1234");
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_fails_to_match_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Matches!(r"\d+");

    let t = Box::<i32>::new(20);
    Box::leak(t);

    assert_eq!(matcher.exec(&parser_context), Err(MatcherFailure::Fail));
  }
}
