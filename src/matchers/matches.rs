extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::token::StandardToken;
use regex::Regex;

#[derive(Debug)]
pub struct MatchesPattern {
  regex: Regex,
  name: String,
  custom_name: bool,
}

impl MatchesPattern {
  pub fn new(regex: Regex) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      regex,
      name: "Matches".to_string(),
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &str, regex: Regex) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      regex,
      name: name.to_string(),
      custom_name: true,
    })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let debug_mode = sub_context.borrow().debug_mode_level();

    if debug_mode > 1 {
      print!("{{Matches}} ");
    }

    let _sc = sub_context.borrow();
    if let Some(range) = _sc.matches_regexp(&self.regex) {
      // We got a match, but it has zero length
      // In this case, respond with a "Skip"
      if range.start == range.end {
        if debug_mode > 0 {
          println!(
            "`{}` Succeeded matching against `{}` -->|{}|--> @[{}-{}] (zero width/skipping)",
            self.get_name(),
            &self.regex,
            _sc
              .debug_range(10)
              .as_str()
              .replace("\n", r"\n")
              .replace("\r", r"\r")
              .replace("\t", r"\t"),
            range.start,
            range.end
          );
        }

        return Ok(MatcherSuccess::Skip(0));
      }

      let token = StandardToken::new(&_sc.parser, self.name.to_string(), range);

      if debug_mode > 0 {
        println!(
          "`{}` Succeeded matching against `{}` -->|{}|--> @[{}-{}]",
          self.get_name(),
          &self.regex,
          token.borrow().get_matched_value(),
          range.start,
          range.end
        );
      }

      Ok(MatcherSuccess::Token(token))
    } else {
      if debug_mode > 0 {
        println!(
          "`{}` Failed to match against `{}` -->|{}|--> @[{}-{}]",
          self.get_name(),
          &self.regex,
          _sc
            .debug_range(10)
            .as_str()
            .replace("\n", r"\n")
            .replace("\r", r"\r")
            .replace("\t", r"\t"),
          _sc.offset.start,
          std::cmp::min(_sc.offset.start + 10, _sc.offset.end),
        );
      }

      Err(MatcherFailure::Fail)
    }
  }
}

impl Matcher for MatchesPattern {
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

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
    self.custom_name = name != "Matches";
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Matches` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
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
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_matches_against_a_regexp() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Matches!(r"\w+");

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Matches");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
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

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Matches");
      assert_eq!(*token.get_captured_range(), SourceRange::new(8, 12));
      assert_eq!(token.get_value(), "1234");
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
