extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::token::StandardToken;

use super::fetch::{Fetchable, FetchableType};

pub struct EqualsPattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  pattern: T,
  name: String,
  custom_name: bool,
}

impl<T> std::fmt::Debug for EqualsPattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EqualsPattern")
      .field("pattern", &self.pattern)
      .field("name", &self.name)
      .field("custom_name", &self.custom_name)
      .finish()
  }
}

impl<T> EqualsPattern<T>
where
  T: Fetchable,
  T: 'static,
  T: std::fmt::Debug,
{
  pub fn new(pattern: T) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      pattern,
      name: "Equals".to_string(),
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &str, pattern: T) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      pattern,
      name: name.to_string(),
      custom_name: true,
    })))
  }
}

impl<T> Matcher for EqualsPattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  fn exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let pattern_value = self.pattern.fetch_value(sub_context.clone(), scope.clone());
    let debug_mode = sub_context.borrow().debug_mode_level();

    match pattern_value {
      FetchableType::String(pattern_value) => {
        if debug_mode > 1 {
          print!("{{Equals}} ");
        }

        let _sc = sub_context.borrow();
        if let Some(range) = _sc.matches_str(pattern_value.as_str()) {
          if debug_mode > 0 {
            println!("`{}` Succeeded matching against `{}` -->|{}|--> @[{}-{}]",
              self.get_name(),
              pattern_value,
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

          Ok(MatcherSuccess::Token(StandardToken::new(
            &_sc.parser,
            self.name.to_string(),
            range,
          )))
        } else {
          if debug_mode > 0 {
            println!(
              "`{}` Failed to match against `{}` -->|{}|--> @[{}-{}]",
              self.get_name(),
              pattern_value,
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
      FetchableType::Matcher(_) => Err(MatcherFailure::Error(
        "`Equals` matcher received another matcher as a pattern... this makes no sense... aborting..."
          .to_string(),
      )),
    }
  }

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
    self.custom_name = true;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Equals` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Equals {
  ($name:expr; $arg:expr) => {
    $crate::matchers::equals::EqualsPattern::new_with_name($name, $arg.to_string())
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
    let matcher = Equals!("testing");

    assert_eq!(
      ParserContext::tokenize(parser_context, matcher),
      Err(MatcherFailure::Fail)
    );
  }
}
