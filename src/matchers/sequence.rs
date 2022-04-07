use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::source_range::SourceRange;
use crate::token::StandardToken;

use super::fetch::{Fetchable, FetchableType};

pub struct SequencePattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  start: T,
  end: T,
  escape: T,
  name: String,
  custom_name: bool,
}

impl<T> std::fmt::Debug for SequencePattern<T>
where
  T: Fetchable,

  T: std::fmt::Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SequencePattern")
      .field("start", &self.start)
      .field("end", &self.end)
      .field("escape", &self.escape)
      .field("name", &self.name)
      .field("custom_name", &self.custom_name)
      .finish()
  }
}

impl<T> SequencePattern<T>
where
  T: Fetchable,
  T: 'static,
  T: std::fmt::Debug,
{
  pub fn new(start: T, end: T, escape: T) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      name: "Sequence".to_string(),
      start,
      end,
      escape,
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &str, start: T, end: T, escape: T) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      name: name.to_string(),
      start,
      end,
      escape,
      custom_name: true,
    })))
  }
}

impl<T> Matcher for SequencePattern<T>
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
    let _sc = sub_context.borrow();

    let start = _sc.offset.start;
    let end = _sc.offset.end;
    let scan_start;
    let source = &_sc.parser.borrow().source[..];
    // let source = source_copy.as_str();

    let debug_mode = _sc.debug_mode_level();

    let start_fetchable = self.start.fetch_value(sub_context.clone(), scope.clone());
    let start_pattern = match start_fetchable {
      FetchableType::String(ref value) => value,
      FetchableType::Matcher(_) => return Err(MatcherFailure::Error(
        "`Sequence` matcher received another matcher as a `start_pattern`... this makes no sense... aborting..."
          .to_string(),
      )),
    };

    if start_pattern.len() == 0 {
      panic!("Sequence `start` pattern of \"\" makes no sense");
    }

    let end_fetchable = self.end.fetch_value(sub_context.clone(), scope.clone());
    let end_pattern = match end_fetchable {
      FetchableType::String(ref value) => value,
      FetchableType::Matcher(_) => return Err(MatcherFailure::Error(
        "`Sequence` matcher received another matcher as a `end_pattern`... this makes no sense... aborting..."
          .to_string(),
      )),
    };
    if end_pattern.len() == 0 {
      panic!("Sequence `end` pattern of \"\" makes no sense");
    }

    let escape_fetchable = self.escape.fetch_value(sub_context.clone(), scope.clone());
    let escape_pattern = match escape_fetchable {
      FetchableType::String(ref value) => value,
      FetchableType::Matcher(_) => return Err(MatcherFailure::Error(
        "`Sequence` matcher received another matcher as a `end_pattern`... this makes no sense... aborting..."
          .to_string(),
      )),
    };

    if debug_mode > 1 {
      print!("{{Sequence}} ");
    }

    if let Some(source_range) = _sc.matches_str(start_pattern) {
      scan_start = source_range.end;
    } else {
      println!(
        "`{}` Failed to match against `{}...{} (escape {})` -->|{}|--> @[{}-{}]",
        self.get_name(),
        start_pattern,
        end_pattern,
        escape_pattern,
        _sc
          .debug_range(10)
          .as_str()
          .replace("\n", r"\n")
          .replace("\r", r"\r")
          .replace("\t", r"\t"),
        _sc.offset.start,
        std::cmp::min(_sc.offset.start + 10, _sc.offset.end),
      );

      return Err(MatcherFailure::Fail);
    }

    let mut index = scan_start;
    let mut previous_index = scan_start;
    let mut parts: Vec<&str> = Vec::new();

    loop {
      if index >= end {
        if debug_mode > 0 {
          println!(
            "`{}` Failed to match against `{}...{} (escape {})` -->|{}|--> @[{}-{}]",
            self.get_name(),
            start_pattern,
            end_pattern,
            escape_pattern,
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

        return Err(MatcherFailure::Fail);
      }

      let result = _sc.matches_str_at_offset(end_pattern, index);
      if let Some(source_range) = result {
        if previous_index < index {
          parts.push(&source[previous_index..index]);
        }

        index = source_range.end;

        let token = StandardToken::new_with_matched_range(
          &_sc.parser,
          self.name.to_string(),
          SourceRange::new(start + start_pattern.len(), index - end_pattern.len()),
          SourceRange::new(start, index),
        );

        token.borrow_mut().set_attribute("__value", &parts.join(""));

        if debug_mode > 0 {
          let _token = token.borrow();
          let range = _token.get_matched_range();
          let full_size = range.end - range.start;

          println!(
            "`{}` Succeeded matching against `{}...{} (escape {})` -->|{}|--> @[{}-{}]",
            self.get_name(),
            start_pattern,
            end_pattern,
            escape_pattern,
            _sc
              .debug_range(full_size)
              .as_str()
              .replace("\n", r"\n")
              .replace("\r", r"\r")
              .replace("\t", r"\t"),
            range.start,
            range.end
          );
        }

        return Ok(MatcherSuccess::Token(token));
      } else {
        let result = _sc.matches_str_at_offset(escape_pattern, index);

        if let Some(source_range) = result {
          if previous_index < index {
            parts.push(&source[previous_index..index]);
          }

          index = source_range.end + 1;
          previous_index = index - 1;

          continue;
        } else {
          index += 1;
          continue;
        }
      }
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
    panic!("Can not add a pattern to a `Sequence` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Sequence {
  ($name:expr; $start:expr, $end:expr, $escape:expr) => {
    $crate::matchers::sequence::SequencePattern::new_with_name($name, $start, $end, $escape)
  };

  ($start:expr, $end:expr, $escape:expr) => {
    $crate::matchers::sequence::SequencePattern::new($start, $end, $escape)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_matches_against_a_sequence() {
    let parser = Parser::new("\"This is a \\\"cool\\\\beans\\\" string!\" stuff after string");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Sequence!("\"", "\"", "\\");

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Sequence");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 34));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 35));
      assert_eq!(token.get_value(), "This is a \"cool\\beans\" string!");
      assert_eq!(
        token.get_matched_value(),
        "\"This is a \\\"cool\\\\beans\\\" string!\""
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_sequence() {
    let parser = Parser::new("\"Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Sequence!("\"", "\"", "\\");

    assert_eq!(
      ParserContext::tokenize(parser_context, matcher),
      Err(MatcherFailure::Fail)
    );
  }
}
