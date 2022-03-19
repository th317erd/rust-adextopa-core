use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct FatalPattern<'a> {
  message: &'a str,
}

impl<'a> FatalPattern<'a> {
  pub fn new(message: &'a str) -> Self {
    Self { message }
  }
}

impl<'a> Matcher for FatalPattern<'a> {
  fn exec(&self, _: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    Err(MatcherFailure::Error(self.message))
  }

  fn get_name(&self) -> &str {
    "Error"
  }
}

#[macro_export]
macro_rules! Fatal {
  ($message:expr) => {
    $crate::matchers::fatal::FatalPattern::new($message)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure},
    parser::Parser,
    parser_context::ParserContext,
    Discard, Matches, Program,
  };

  #[test]
  fn it_can_throw_a_fatal_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Matches!(r"\w+"),
      Fatal!("There was an error!"),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+")
    );

    if let Err(MatcherFailure::Error(message)) = matcher.exec(parser_context.clone()) {
      assert_eq!(message, "There was an error!");
    } else {
      unreachable!("Test failed!");
    };
  }
}
