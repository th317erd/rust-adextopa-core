use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef};

pub fn new_error_token(context: ParserContextRef, message: &str) -> TokenRef {
  let context = context.borrow();
  let captured_range = SourceRange::new(context.offset.start, context.offset.start);
  let token = StandardToken::new(&context.parser, "Error".to_string(), captured_range);

  {
    let mut token = token.borrow_mut();
    token.set_attribute("__message", message);
    token.set_attribute("__is_error", "true");
  }

  token
}

pub fn new_error_token_with_range(
  context: ParserContextRef,
  message: &str,
  matched_range: SourceRange,
) -> TokenRef {
  let context = context.borrow();
  let token = StandardToken::new(&context.parser, "Error".to_string(), matched_range);

  {
    let mut token = token.borrow_mut();
    token.set_attribute("__message", message);
    token.set_attribute("__is_error", "true");
  }

  token
}

#[derive(Debug)]
pub struct ErrorPattern<'a> {
  message: &'a str,
}

impl<'a> ErrorPattern<'a> {
  pub fn new(message: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { message })))
  }
}

impl<'a> Matcher<'a> for ErrorPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let error_token = new_error_token(context, self.message);
    Ok(MatcherSuccess::Token(error_token))
  }

  fn get_name(&self) -> &str {
    "Error"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Error` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Error` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Error {
  ($message:expr) => {
    $crate::matchers::error::ErrorPattern::new($message)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Discard, Matches, Program,
  };

  #[test]
  fn it_can_record_an_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Matches!(r"\w+"),
      Error!("There was an error!"),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+")
    );

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_captured_value(), "Testing 1234");
      assert_eq!(token.get_matched_value(), "Testing 1234");
      assert_eq!(token.get_children().len(), 3);

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Error");
      assert_eq!(*second.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*second.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(second.get_captured_value(), "Testing");
      assert_eq!(second.get_matched_value(), "Testing");
      assert_eq!(
        second.get_attribute("__message").unwrap(),
        "There was an error!"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
