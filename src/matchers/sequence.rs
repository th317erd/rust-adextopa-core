use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::StandardToken;

pub struct SequencePattern<'a> {
  start: &'a str,
  end: &'a str,
  escape: &'a str,
  name: &'a str,
}

impl<'a> SequencePattern<'a> {
  pub fn new(start: &'a str, end: &'a str, escape: &'a str) -> MatcherRef<'a> {
    if start.len() == 0 {
      panic!("Sequence start pattern of \"\" makes no sense");
    }

    if end.len() == 0 {
      panic!("Sequence end pattern of \"\" makes no sense");
    }

    Rc::new(RefCell::new(Box::new(Self {
      name: "Sequence",
      start,
      end,
      escape,
    })))
  }

  pub fn new_with_name(
    name: &'a str,
    start: &'a str,
    end: &'a str,
    escape: &'a str,
  ) -> MatcherRef<'a> {
    if start.len() == 0 {
      panic!("Sequence start pattern of \"\" makes no sense");
    }

    if end.len() == 0 {
      panic!("Sequence end pattern of \"\" makes no sense");
    }

    Rc::new(RefCell::new(Box::new(Self {
      name,
      start,
      end,
      escape,
    })))
  }
}

impl<'a> Matcher<'a> for SequencePattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let start = context.borrow().offset.start;
    let end = context.borrow().offset.end;
    let scan_start;
    let source_copy = context.borrow().parser.borrow().source.clone();
    let source = source_copy.as_str();

    if let Some(source_range) = context.borrow().matches_str(self.start) {
      scan_start = source_range.end;
    } else {
      return Err(MatcherFailure::Fail);
    }

    if scan_start >= end {
      return Err(MatcherFailure::Fail);
    }

    let mut index = scan_start;
    let mut previous_index = scan_start;
    let mut parts: Vec<&str> = Vec::new();

    loop {
      if index >= end {
        return Err(MatcherFailure::Fail);
      }

      let result = context.borrow().matches_str_at_offset(self.end, index);
      if let Some(source_range) = result {
        if previous_index < index {
          parts.push(&source[previous_index..index]);
        }

        index = source_range.end;

        context.borrow_mut().set_start(index);

        let token = StandardToken::new_with_raw_range(
          &context.borrow().parser,
          self.name.to_string(),
          SourceRange::new(start + self.start.len(), index - self.end.len()),
          SourceRange::new(start, index),
        );

        token
          .borrow_mut()
          .set_attribute("__value".to_string(), parts.join("").to_string());

        return Ok(MatcherSuccess::Token(token));
      } else {
        let result = context.borrow().matches_str_at_offset(self.escape, index);
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
    panic!("Can not add a pattern to a Sequence pattern");
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

    if let Ok(MatcherSuccess::Token(token)) = matcher.borrow().exec(parser_context.clone()) {
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
      matcher.borrow().exec(parser_context.clone()),
      Err(MatcherFailure::Fail)
    );
  }
}
