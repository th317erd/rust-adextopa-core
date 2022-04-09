use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope::VariableType;
use crate::scope_context::ScopeContextRef;

#[derive(Debug)]
pub struct RegisterPattern {
  patterns: Vec<MatcherRef>,
}

impl RegisterPattern {
  pub fn new_blank() -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      patterns: Vec::new(),
    })))
  }

  fn _exec(
    &self,
    _: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let mut scope_mut = scope.borrow_mut();
    for pattern in &self.patterns {
      scope_mut.set(
        pattern.borrow().get_name(),
        VariableType::Matcher(pattern.clone()),
      );
    }

    // Always skip... as this is a "no-op" pattern
    Ok(MatcherSuccess::Skip(0))
  }
}

impl Matcher for RegisterPattern {
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

  fn is_consuming(&self) -> bool {
    false
  }

  fn get_name(&self) -> &str {
    "Register"
  }

  fn set_name(&mut self, _: &str) {
    // panic!("Can not set `name` on a `Register` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    Some(self.patterns.clone())
  }

  fn add_pattern(&mut self, pattern: MatcherRef) {
    self.patterns.push(pattern);
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Register {
  ($($args:expr),+ $(,)?) => {
    {
      let register = $crate::matchers::register::RegisterPattern::new_blank();

      {
        let mut rm = register.borrow_mut();

        $(
          rm.add_pattern($args);
        )*
      }

      register
    }
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    parser::Parser, parser_context::ParserContext, source_range::SourceRange, Matches, Program, Ref,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Hello World");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Register!(Matches!("Word"; r"\w+")),
      Ref!("Word"),
      Matches!(r"\s+"),
      Ref!("Word")
    );

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_value(), "Hello World");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Word");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(first.get_value(), "Hello");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Matches");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 6));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 6));
      assert_eq!(second.get_value(), " ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Word");
      assert_eq!(*third.get_captured_range(), SourceRange::new(6, 11));
      assert_eq!(*third.get_matched_range(), SourceRange::new(6, 11));
      assert_eq!(third.get_value(), "World");
    } else {
      unreachable!("Test failed!");
    };
  }
}
