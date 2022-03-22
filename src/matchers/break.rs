use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct BreakPattern<'a> {
  loop_name: &'a str,
}

impl<'a> BreakPattern<'a> {
  pub fn new(loop_name: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(BreakPattern { loop_name })))
  }
}

impl<'a> Matcher<'a> for BreakPattern<'a> {
  fn exec(&self, _: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Break((
      self.loop_name.to_string(),
      Box::new(MatcherSuccess::None),
    )))
  }

  fn get_name(&self) -> &str {
    "Break"
  }

  fn set_name(&mut self, _: &'a str) {
    panic!("Can not set `name` on a `Break` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Break` matcher");
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
    let result = ParserContext::tokenize(parser_context, matcher);

    assert_eq!(
      result,
      Ok(MatcherSuccess::Break((
        "Test".to_string(),
        Box::new(MatcherSuccess::None),
      )))
    );
  }
}
