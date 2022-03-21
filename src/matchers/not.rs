use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct NotPattern<'a> {
  matcher: MatcherRef<'a>,
}

impl<'a> NotPattern<'a> {
  pub fn new(matcher: MatcherRef<'a>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
  }
}

impl<'a> Matcher<'a> for NotPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match self
      .matcher
      .borrow()
      .exec(context.borrow().clone_with_name(self.get_name()))
    {
      Ok(success) => match success {
        // Fail on success
        MatcherSuccess::Token(_) => return Err(MatcherFailure::Fail),
        MatcherSuccess::ExtractChildren(_) => return Err(MatcherFailure::Fail),
        MatcherSuccess::Skip(amount) => {
          // If Skip value is anything but zero, then fail
          if amount != 0 {
            return Err(MatcherFailure::Fail);
          }

          return Ok(success);
        }
        // For other success types (Skip, Stop, Break, Continue, None) succeed
        _ => return Ok(success),
      },
      Err(failure) => match failure {
        // Succeed on fail
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(err) => Err(MatcherFailure::Error(err)),
      },
    }
  }

  fn get_name(&self) -> &str {
    "Not"
  }

  fn set_name(&mut self, _: &'a str) {
    panic!("Can not set 'name' on a Not pattern");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    Some(vec![self.matcher.clone()])
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a Not pattern");
  }
}

#[macro_export]
macro_rules! Not {
  ($arg:expr) => {
    $crate::matchers::not::NotPattern::new($arg.clone())
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Not!(Equals!("Testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      matcher.borrow().exec(parser_context.clone())
    );
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Not!(Equals!("testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      matcher.borrow().exec(parser_context.clone())
    );
  }
}
