use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CatchPattern<F>
where
  F: Fn(ParserContextRef, MatcherFailure) -> Result<MatcherSuccess, MatcherFailure>,
{
  matcher: MatcherRef,
  catch_func: F,
}

impl<F> std::fmt::Debug for CatchPattern<F>
where
  F: Fn(ParserContextRef, MatcherFailure) -> Result<MatcherSuccess, MatcherFailure>,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CatchPattern")
      .field("matcher", &self.matcher)
      .finish()
  }
}

impl<F> CatchPattern<F>
where
  F: Fn(ParserContextRef, MatcherFailure) -> Result<MatcherSuccess, MatcherFailure>,
  F: 'static,
{
  pub fn new(matcher: MatcherRef, catch_func: F) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      matcher,
      catch_func,
    })))
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
      Ok(success) => Ok(success),
      Err(failure) => (self.catch_func)(sub_context.clone(), failure),
    }
  }
}

impl<F> Matcher for CatchPattern<F>
where
  F: Fn(ParserContextRef, MatcherFailure) -> Result<MatcherSuccess, MatcherFailure>,
  F: 'static,
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
    "Catch"
  }

  fn set_name(&mut self, name: &str) {
    // panic!("Can not set `name` on a `Catch` matcher");
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
    panic!("Can not add a pattern to a `Catch` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Catch {
  ($matcher:expr, $catch_func:expr) => {
    $crate::matchers::catch::CatchPattern::new($matcher, $catch_func)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parse_error::ParseError, parser::Parser,
    parser_context::ParserContext, source_range::SourceRange, Equals, Panic,
  };

  #[test]
  fn it_does_nothing_on_success() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Catch!(Equals!("Testing"), |_, _| {
      unreachable!("Test failed!");
    });

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_catch_a_failure() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Catch!(Equals!("Derp"), |context, _| {
      Err(MatcherFailure::Error(ParseError::new_with_range(
        "There was a big fat error!",
        context.borrow().offset,
      )))
    });

    if let Err(MatcherFailure::Error(failure)) = ParserContext::tokenize(parser_context, matcher) {
      assert_eq!(failure.message, "There was a big fat error!");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_catch_an_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Catch!(Panic!("Holy malarky! I failed!"), |context, failure| {
      Err(MatcherFailure::Error(ParseError::new_with_range(
        &format!("There was a big fat error!: {:?}", failure),
        context.borrow().offset,
      )))
    });

    if let Err(MatcherFailure::Error(failure)) = ParserContext::tokenize(parser_context, matcher) {
      assert_eq!(
        failure.message,
        "There was a big fat error!: Error(ParseError { message: \"Error: @[1:1]: Holy malarky! I failed!\", range: Some(SourceRange { start: 0, end: 0 }) })"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
