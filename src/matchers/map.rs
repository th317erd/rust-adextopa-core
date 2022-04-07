use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::token::TokenRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MapPattern<F>
where
  F: Fn(TokenRef) -> Option<String>,
{
  matcher: MatcherRef,
  map_func: F,
}

impl<F> std::fmt::Debug for MapPattern<F>
where
  F: Fn(TokenRef) -> Option<String>,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MapPattern")
      .field("matcher", &self.matcher)
      .finish()
  }
}

impl<F> MapPattern<F>
where
  F: Fn(TokenRef) -> Option<String>,
  F: 'static,
{
  pub fn new(matcher: MatcherRef, map_func: F) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self { matcher, map_func })))
  }
}

impl<F> Matcher for MapPattern<F>
where
  F: Fn(TokenRef) -> Option<String>,
{
  fn exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let result = self.matcher.borrow().exec(
      context.borrow().clone_with_name(self.get_name()),
      scope.clone(),
    );

    match result {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          if let Some(result) = (self.map_func)(token.clone()) {
            return Ok(MatcherSuccess::Token(
              crate::matchers::error::new_error_token_with_range(
                context,
                result.as_str(),
                token.borrow().get_matched_range().clone(),
              ),
            ));
          }

          Ok(MatcherSuccess::Token(token))
        }
        _ => Ok(success),
      },
      Err(failure) => Err(failure),
    }
  }

  fn get_name(&self) -> &str {
    "Map"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Map` matcher");
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
    panic!("Can not add a pattern to a `Map` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Map {
  ($matcher:expr, $map_func:expr) => {
    $crate::matchers::map::MapPattern::new($matcher, $map_func)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals,
  };

  #[test]
  fn it_can_mutate_a_token() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Map!(Equals!("Testing"), |token| {
      let captured_range = token.borrow().get_captured_range().clone();
      let mut token = token.borrow_mut();

      token.set_name("WOW");
      token.set_captured_range(SourceRange::new(
        captured_range.start + 1,
        captured_range.end - 1,
      ));
      token.set_attribute("was_mapped", "true");

      None
    });

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "WOW");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "estin");
      assert_eq!(token.get_matched_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_return_an_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Map!(Equals!("Testing"), |_| {
      Some("There was a big fat error!".to_string())
    });

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "Testing");
      assert_eq!(
        token.get_attribute("__message").unwrap(),
        "There was a big fat error!"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Map!(Equals!("testing"), |_| { None });

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
