use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::{ParserContext, ParserContextRef};

pub struct BreakPattern<'a> {
  loop_name: &'a str,
}

impl<'a> BreakPattern<'a> {
  pub fn new(loop_name: &'a str) -> Self {
    BreakPattern { loop_name }
  }
}

impl<'a> Matcher for BreakPattern<'a> {
  fn exec(&self, _: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Break((
      self.loop_name,
      Box::new(MatcherSuccess::None),
    )))
  }

  fn get_name(&self) -> &str {
    "Break"
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

mod tests {
  use crate::{
    matcher::{Matcher, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Break,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Break!("Test");

    assert_eq!(
      matcher.exec(parser_context.clone()),
      Ok(MatcherSuccess::Break((
        "Test",
        Box::new(MatcherSuccess::None),
      )))
    );
  }
}
