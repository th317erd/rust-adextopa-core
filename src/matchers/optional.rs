use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct OptionalPattern<'a> {
  matcher: MatcherRef<'a>,
}

impl<'a> OptionalPattern<'a> {
  pub fn new(matcher: MatcherRef<'a>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
  }
}

impl<'a> Matcher<'a> for OptionalPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match self
      .matcher
      .borrow()
      .exec(context.borrow().clone_with_name(self.get_name()))
    {
      Ok(success) => Ok(success),
      Err(failure) => match failure {
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(error) => Err(MatcherFailure::Error(error)),
      },
    }
  }

  fn get_name(&self) -> &str {
    "Optional"
  }

  fn set_name(&mut self, _: &'a str) {
    panic!("Can not set `name` on a `Optional` matcher");
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
    panic!("Can not add a pattern to a `Optional` matcher");
  }
}

#[macro_export]
macro_rules! Optional {
  ($arg:expr) => {
    $crate::matchers::optional::OptionalPattern::new($arg.clone())
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Optional!(Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Optional!(Equals!("testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
