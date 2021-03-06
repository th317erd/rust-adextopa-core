use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct ProxyChildrenPattern {
  matcher: MatcherRef,
  name: String,
  custom_name: bool,
}

impl ProxyChildrenPattern {
  pub fn new(matcher: MatcherRef) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(ProxyChildrenPattern {
      matcher,
      name: "Flatten".to_string(),
      custom_name: false,
    })))
  }

  pub fn new_with_name(matcher: MatcherRef, name: &str) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(ProxyChildrenPattern {
      matcher,
      name: name.to_string(),
      custom_name: true,
    })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let result = self.matcher.borrow().exec(
      self.matcher.clone(),
      context.borrow().clone_with_name(self.get_name()),
      scope.clone(),
    );

    match result {
      Ok(MatcherSuccess::Token(token)) => Ok(MatcherSuccess::ProxyChildren(token.clone())),
      Ok(MatcherSuccess::Break((loop_name, value))) => match *value {
        MatcherSuccess::Token(token) => Ok(MatcherSuccess::Break((
          loop_name,
          Box::new(MatcherSuccess::ProxyChildren(token.clone())),
        ))),
        _ => Ok(MatcherSuccess::Break((loop_name, value))),
      },
      Ok(MatcherSuccess::Continue((loop_name, value))) => match *value {
        MatcherSuccess::Token(token) => Ok(MatcherSuccess::Continue((
          loop_name,
          Box::new(MatcherSuccess::ProxyChildren(token.clone())),
        ))),
        _ => Ok(MatcherSuccess::Continue((loop_name, value))),
      },
      _ => result,
    }
  }
}

impl Matcher for ProxyChildrenPattern {
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

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
    self.custom_name = true;
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
    panic!("Can not add a pattern to a `Flatten` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! ProxyChildren {
  ($name:literal; $arg:expr) => {
    $crate::matchers::proxy_children::ProxyChildrenPattern::new_with_name($arg, $name)
  };

  ($arg:expr) => {
    $crate::matchers::proxy_children::ProxyChildrenPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    parser::Parser, parser_context::ParserContext, source_range::SourceRange, Loop, Matches,
    ProxyChildren, Switch,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(
      "Loop";
      ProxyChildren!(Loop!(
        Switch!(
          Matches!("Whitespace"; r"\s+"),
          Matches!("Word"; r"[a-zA-Z_]\w+"),
          Matches!("Number"; r"\d+")
        )
      ))
    );

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), "Testing 1234");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Word");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Whitespace");
      assert_eq!(*second.get_captured_range(), SourceRange::new(7, 8));
      assert_eq!(*second.get_matched_range(), SourceRange::new(7, 8));
      assert_eq!(second.get_value(), " ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Number");
      assert_eq!(*third.get_captured_range(), SourceRange::new(8, 12));
      assert_eq!(*third.get_matched_range(), SourceRange::new(8, 12));
      assert_eq!(third.get_value(), "1234");
    } else {
      unreachable!("Test failed!");
    };
  }
}
