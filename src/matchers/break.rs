use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;

#[derive(Debug)]
pub struct BreakPattern {
  loop_name: String,
}

impl BreakPattern {
  pub fn new(loop_name: &str) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(BreakPattern {
      loop_name: loop_name.to_string(),
    })))
  }
}

impl Matcher for BreakPattern {
  fn exec(
    &self,
    _: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Break((
      self.loop_name.to_string(),
      Box::new(MatcherSuccess::None),
    )))
  }

  fn get_name(&self) -> &str {
    "Break"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Break` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Break` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Break {
  ($arg:expr) => {
    $crate::matchers::r#break::BreakPattern::new($arg)
  };

  () => {
    $crate::matchers::r#break::BreakPattern::new("")
  };
}

#[cfg(test)]
mod tests {
  use crate::{matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext, Break};

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Break!("Test");
    let result = matcher.borrow().exec(
      parser_context.clone(),
      parser_context.borrow().scope.clone(),
    );

    assert_eq!(
      result,
      Ok(MatcherSuccess::Break((
        "Test".to_string(),
        Box::new(MatcherSuccess::None),
      )))
    );
  }
}
