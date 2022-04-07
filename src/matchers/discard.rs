use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef};

#[derive(Debug)]
pub struct DiscardPattern {
  matcher: MatcherRef,
}

impl DiscardPattern {
  pub fn new(matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
  }
}

fn collect_errors(error_token: TokenRef, walk_token: TokenRef) {
  let is_error = if let Some(value) = walk_token.borrow().get_attribute("__is_error") {
    value == "true"
  } else {
    false
  };

  if is_error {
    let mut error_token = error_token.borrow_mut();
    error_token.get_children_mut().push(walk_token.clone());

    let walk_token = walk_token.borrow();
    let walk_token_range = walk_token.get_matched_range();
    let error_token_range = error_token.get_matched_range();
    let mut new_start = error_token_range.start;
    let mut new_end = error_token_range.end;

    if new_start > walk_token_range.start {
      new_start = walk_token_range.start;
    }

    if new_end < walk_token_range.end {
      new_end = walk_token_range.end;
    }

    if new_start != error_token_range.start || new_end != error_token_range.end {
      let new_source_range = SourceRange::new(new_start, new_end);
      error_token.set_matched_range(new_source_range);
    }
  }

  {
    let walk_token = walk_token.borrow();
    let children = walk_token.get_children();
    for child in children {
      collect_errors(error_token.clone(), child.clone());
    }
  }
}

fn skip_token(context: ParserContextRef, start_offset: usize, token: TokenRef) -> MatcherSuccess {
  let end_offset = token.borrow().get_matched_range().end;

  // Check to see if any errors are in the result
  // If there are, continue to proxy them upstream
  let error_token = StandardToken::new(
    &context.borrow().parser,
    "Error".to_string(),
    SourceRange::new(start_offset, end_offset),
  );
  collect_errors(error_token.clone(), token.clone());

  if error_token.borrow().get_children().len() > 0 {
    return MatcherSuccess::Token(error_token);
  }

  let offset: isize = end_offset as isize - start_offset as isize;
  MatcherSuccess::Skip(offset)
}

impl Matcher for DiscardPattern {
  fn exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let start_offset = context.offset.start;
    let sub_context = context.clone_with_name(self.get_name());

    match self
      .matcher
      .borrow()
      .exec(sub_context.clone(), scope.clone())
    {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          return Ok(skip_token(sub_context, start_offset, token.clone()));
        }
        MatcherSuccess::ExtractChildren(token) => {
          return Ok(skip_token(sub_context, start_offset, token.clone()));
        }
        MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Skip(offset)),
        // Here we Discard "Break"s payload
        MatcherSuccess::Break((loop_name, result)) => match *result {
          MatcherSuccess::Token(token) => Ok(MatcherSuccess::Break((
            loop_name,
            Box::new(skip_token(sub_context, start_offset, token.clone())),
          ))),
          MatcherSuccess::ExtractChildren(token) => Ok(MatcherSuccess::Break((
            loop_name,
            Box::new(skip_token(sub_context, start_offset, token.clone())),
          ))),
          MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Break((
            loop_name,
            Box::new(MatcherSuccess::Skip(offset)),
          ))),
          MatcherSuccess::None => Ok(MatcherSuccess::Break((
            loop_name,
            Box::new(MatcherSuccess::None),
          ))),
          MatcherSuccess::Stop => Ok(MatcherSuccess::Break((
            loop_name,
            Box::new(MatcherSuccess::Stop),
          ))),
          _ => unreachable!(),
        },
        // Here we Discard "Continue"s payload
        MatcherSuccess::Continue((loop_name, result)) => match *result {
          MatcherSuccess::Token(token) => Ok(MatcherSuccess::Continue((
            loop_name,
            Box::new(skip_token(sub_context, start_offset, token.clone())),
          ))),
          MatcherSuccess::ExtractChildren(token) => Ok(MatcherSuccess::Continue((
            loop_name,
            Box::new(skip_token(sub_context, start_offset, token.clone())),
          ))),
          MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Continue((
            loop_name,
            Box::new(MatcherSuccess::Skip(offset)),
          ))),
          MatcherSuccess::None => Ok(MatcherSuccess::Continue((
            loop_name,
            Box::new(MatcherSuccess::None),
          ))),
          MatcherSuccess::Stop => Ok(MatcherSuccess::Continue((
            loop_name,
            Box::new(MatcherSuccess::Stop),
          ))),
          _ => unreachable!(),
        },
        _ => Ok(success),
      },
      Err(failure) => {
        return Err(failure);
      }
    }
  }

  fn get_name(&self) -> &str {
    "Discard"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Discard` matcher");
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef) {
    if index > 0 {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.matcher = matcher;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    Some(vec![self.matcher.clone()])
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Discard` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Discard {
  ($arg:expr) => {
    $crate::matchers::discard::DiscardPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals, Error, Program,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Discard!(Equals!("Testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(7)),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_properly_returns_error_tokens() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Discard!(Program!(Equals!("Testing"), Error!("This is an error")));

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "Testing");
      assert_eq!(token.get_children().len(), 1);
      assert_eq!(token.get_attribute("__message"), None);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Error");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "Testing");
      assert_eq!(first.get_matched_value(), "Testing");
      assert_eq!(first.get_children().len(), 0);
      assert_eq!(
        first.attribute_equals("__message", "This is an error"),
        true
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Discard!(Equals!("testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
