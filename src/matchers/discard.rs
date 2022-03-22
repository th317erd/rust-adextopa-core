use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef};

pub struct DiscardPattern<'a> {
  matcher: MatcherRef<'a>,
}

impl<'a> DiscardPattern<'a> {
  pub fn new(matcher: MatcherRef<'a>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
  }
}

fn collect_errors<'a, 'b>(error_token: TokenRef, walk_token: TokenRef) {
  if walk_token.borrow().get_name() == "Error" {
    error_token
      .borrow_mut()
      .get_children_mut()
      .push(walk_token.clone());
  }

  {
    let walk_token = walk_token.borrow();
    let children = walk_token.get_children();
    if children.len() > 0 {
      for child in children {
        collect_errors(error_token.clone(), child.clone());
      }
    }
  }
}

impl<'a> Matcher<'a> for DiscardPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let sub_context = context.clone_with_name(self.get_name());
    let start_offset = context.offset.start;

    match self.matcher.borrow().exec(sub_context.clone()) {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          let end_offset = token.borrow().get_raw_range().end;
          let offset: isize = end_offset as isize - start_offset as isize;

          // Check to see if any errors are in the result
          // If there are, continue to proxy them upstream
          let error_token = StandardToken::new(
            &context.parser,
            "Error".to_string(),
            SourceRange::new(start_offset, end_offset),
          );
          collect_errors(error_token.clone(), token.clone());

          if error_token.borrow().get_children().len() > 0 {
            return Ok(MatcherSuccess::Token(error_token));
          }

          return Ok(MatcherSuccess::Skip(offset));
        }
        MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Skip(offset)),
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

  fn set_name(&mut self, _: &'a str) {
    panic!("Can not set 'name' on a Discard pattern");
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef<'a>) {
    if index > 0 {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.matcher = matcher;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    Some(vec![self.matcher.clone()])
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a Discard pattern");
  }
}

#[macro_export]
macro_rules! Discard {
  ($arg:expr) => {
    $crate::matchers::discard::DiscardPattern::new($arg.clone())
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
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");
      assert_eq!(token.get_children().len(), 1);
      assert_eq!(token.get_attribute("__message".to_string()), None);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Error");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(first.value(), "Testing");
      assert_eq!(first.raw_value(), "Testing");
      assert_eq!(first.get_children().len(), 0);
      assert_eq!(
        first.get_attribute("__message".to_string()),
        Some(&"This is an error".to_string())
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
