use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::token::TokenRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MapPattern<SF, FF>
where
  SF: Fn(TokenRef, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  FF:
    Fn(MatcherFailure, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
{
  matcher: MatcherRef,
  success_func: SF,
  failure_func: Option<FF>,
}

impl<SF, FF> std::fmt::Debug for MapPattern<SF, FF>
where
  SF: Fn(TokenRef, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  FF:
    Fn(MatcherFailure, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MapPattern")
      .field("matcher", &self.matcher)
      .finish()
  }
}

impl<SF, FF> MapPattern<SF, FF>
where
  SF: Fn(TokenRef, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  FF:
    Fn(MatcherFailure, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  SF: 'static,
  FF: 'static,
{
  pub fn new(matcher: MatcherRef, success_func: SF, failure_func: FF) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      matcher,
      success_func,
      failure_func: Some(failure_func),
    })))
  }

  fn handle_success(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
    token: TokenRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    (self.success_func)(token.clone(), context.clone(), scope.clone())
  }

  fn handle_failure(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
    failure: MatcherFailure,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    if self.failure_func.is_none() {
      return Err(failure);
    }

    (self.failure_func.as_ref().unwrap())(failure, context.clone(), scope.clone())
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let result =
      self
        .matcher
        .borrow()
        .exec(self.matcher.clone(), sub_context.clone(), scope.clone());

    match result {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          self.handle_success(sub_context.clone(), scope.clone(), token.clone())
        }
        MatcherSuccess::ExtractChildren(token) => {
          self.handle_success(sub_context.clone(), scope.clone(), token.clone())
        }
        _ => Ok(success),
      },
      Err(failure) => self.handle_failure(sub_context.clone(), scope.clone(), failure),
    }
  }
}

impl<SF, FF> Matcher for MapPattern<SF, FF>
where
  SF: Fn(TokenRef, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  FF:
    Fn(MatcherFailure, ParserContextRef, ScopeContextRef) -> Result<MatcherSuccess, MatcherFailure>,
  SF: 'static,
  FF: 'static,
{
  fn exec(
    &self,
    this_matcher: MatcherRef,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    self.before_exec(this_matcher.clone(), context.clone(), scope.clone());
    let result = self._exec(context.clone(), scope.clone());
    self.after_exec(this_matcher.clone(), context.clone(), scope.clone());

    result
  }

  fn get_name(&self) -> &str {
    "Map"
  }

  fn set_name(&mut self, name: &str) {
    // panic!("Can not set `name` on a `Map` matcher");
    self.matcher.borrow_mut().set_name(name);
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
  ($matcher:expr, $success_func:expr, $failure_func:expr) => {
    $crate::matchers::map::MapPattern::new($matcher, $success_func, $failure_func)
  };

  ($matcher:expr, $success_func:expr) => {
    $crate::matchers::map::MapPattern::new($matcher, $success_func, |failure, _, __| Err(failure))
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Equals, ErrorTokenResult, TokenResult,
  };

  #[test]
  fn it_can_mutate_a_token() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Map!(Equals!("Testing"), |token, _, __| {
      let captured_range = token.borrow().get_captured_range().clone();
      let mut _token = token.borrow_mut();

      _token.set_name("WOW");
      _token.set_captured_range(SourceRange::new(
        captured_range.start + 1,
        captured_range.end - 1,
      ));
      _token.set_attribute("was_mapped", "true");

      TokenResult!(token.clone())
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
    let matcher = Map!(Equals!("Testing"), |token, context, ___| {
      ErrorTokenResult!(
        context.clone(),
        "There was a big fat error!",
        &token.borrow().get_matched_range()
      )
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
        "Error: @[1:1-8]: There was a big fat error!"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Map!(Equals!("testing"), |token, _, __| {
      TokenResult!(token.clone())
    });

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
