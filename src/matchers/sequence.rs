use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::StandardToken;

use super::fetch::Fetchable;

pub struct SequencePattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  start: T,
  end: T,
  escape: T,
  name: &'a str,
  custom_name: bool,
}

impl<'a, T> SequencePattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  pub fn new(start: T, end: T, escape: T) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      name: "Sequence",
      start,
      end,
      escape,
      custom_name: false,
    })))
  }

  pub fn new_with_name(name: &'a str, start: T, end: T, escape: T) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      name,
      start,
      end,
      escape,
      custom_name: true,
    })))
  }
}

impl<'a, T> Matcher<'a> for SequencePattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
{
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let start = context.borrow().offset.start;
    let end = context.borrow().offset.end;
    let scan_start;
    let source_copy = context.borrow().parser.borrow().source.clone();
    let source = source_copy.as_str();

    let sub_context = context.borrow().clone_with_name(self.get_name());

    let start_fetchable = self.start.fetch_value(sub_context.clone());
    let start_pattern = start_fetchable.as_str();

    if start_pattern.len() == 0 {
      panic!("Sequence `start` pattern of \"\" makes no sense");
    }

    if let Some(source_range) = context.borrow().matches_str(start_pattern) {
      scan_start = source_range.end;

      if scan_start >= end {
        return Err(MatcherFailure::Fail);
      }
    } else {
      return Err(MatcherFailure::Fail);
    }

    let end_fetchable = self.end.fetch_value(sub_context.clone());
    let end_pattern = end_fetchable.as_str();
    if end_pattern.len() == 0 {
      panic!("Sequence `end` pattern of \"\" makes no sense");
    }

    let escape_fetchable = self.escape.fetch_value(sub_context);
    let escape_pattern = escape_fetchable.as_str();

    let mut index = scan_start;
    let mut previous_index = scan_start;
    let mut parts: Vec<&str> = Vec::new();

    loop {
      if index >= end {
        return Err(MatcherFailure::Fail);
      }

      let result = context.borrow().matches_str_at_offset(end_pattern, index);
      if let Some(source_range) = result {
        if previous_index < index {
          parts.push(&source[previous_index..index]);
        }

        index = source_range.end;

        context.borrow_mut().set_start(index);

        let token = StandardToken::new_with_raw_range(
          &context.borrow().parser,
          self.name.to_string(),
          SourceRange::new(start + start_pattern.len(), index - end_pattern.len()),
          SourceRange::new(start, index),
        );

        token.borrow_mut().set_attribute("__value", &parts.join(""));

        return Ok(MatcherSuccess::Token(token));
      } else {
        let result = context
          .borrow()
          .matches_str_at_offset(escape_pattern, index);

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
    panic!("Can not add a pattern to a `Sequence` matcher");
  }
}

#[macro_export]
macro_rules! Sequence {
  ($name:literal; $start:expr, $end:expr, $escape:expr) => {
    $crate::matchers::sequence::SequencePattern::new_with_name($name, $start, $end, $escape)
  };

  ($start:expr, $end:expr, $escape:expr) => {
    $crate::matchers::sequence::SequencePattern::new($start, $end, $escape)
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
  fn it_matches_against_a_sequence() {
    let parser = Parser::new("\"This is a \\\"cool\\\\beans\\\" string!\" stuff after string");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Sequence!("\"", "\"", "\\");

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Sequence");
      assert_eq!(*token.get_value_range(), SourceRange::new(1, 34));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 35));
      assert_eq!(token.value(), "This is a \"cool\\beans\" string!");
      assert_eq!(
        token.raw_value(),
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
