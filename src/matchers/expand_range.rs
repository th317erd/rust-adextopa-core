use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ExpandRange {
  matcher: MatcherRef,
}

impl std::fmt::Debug for ExpandRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExpandRange")
      .field("matcher", &self.matcher)
      .finish()
  }
}

impl ExpandRange {
  pub fn new(matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self { matcher })))
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
          let matched_range = token.borrow().get_matched_range().clone();
          token.borrow_mut().set_captured_range(matched_range);

          Ok(MatcherSuccess::Token(token))
        }
        MatcherSuccess::ProxyChildren(token) => {
          let matched_range = token.borrow().get_matched_range().clone();
          token.borrow_mut().set_captured_range(matched_range);

          Ok(MatcherSuccess::ProxyChildren(token))
        }
        _ => Ok(success),
      },
      _ => result,
    }
  }
}

impl Matcher for ExpandRange {
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
macro_rules! ExpandRange {
  ($matcher:expr) => {
    $crate::matchers::expand_range::ExpandRange::new($matcher)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Panic, Sequence,
  };

  #[test]
  fn it_does_nothing_on_success() {
    let parser = Parser::new("'Testing' 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ExpandRange!(Sequence!("'", "'", "\\"));

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Sequence");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 9));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 9));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "'Testing'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_fail() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ExpandRange!(Panic!("Holy malarky! I failed!"));

    if let Err(MatcherFailure::Error(failure)) = ParserContext::tokenize(parser_context, matcher) {
      assert_eq!(failure.message, "Error: @[1:1]: Holy malarky! I failed!");
    } else {
      unreachable!("Test failed!");
    };
  }
}
