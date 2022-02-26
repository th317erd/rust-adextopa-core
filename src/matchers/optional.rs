use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContext;

pub struct OptionalPattern {
  matcher: Box<dyn Matcher>,
}

impl OptionalPattern {
  pub fn new(matcher: Box<dyn Matcher>) -> Self {
    Self { matcher }
  }
}

impl Matcher for OptionalPattern {
  fn exec(&self, context: &ParserContext) -> Result<MatcherSuccess, MatcherFailure> {
    match self.matcher.exec(context) {
      Ok(success) => Ok(success),
      Err(failure) => match failure {
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(error) => Err(MatcherFailure::Error(error)),
      },
    }
  }

  fn get_name(&self) -> &str {
    "Optional"
  }
}

#[macro_export]
macro_rules! Optional {
  ($arg:expr) => {
    crate::matchers::optional::OptionalPattern::new(Box::new($arg))
  };
}

mod tests {
  use crate::{
    matcher::{Matcher, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Optional!(Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(&parser), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Optional!(Equals!("testing"));

    assert_eq!(Ok(MatcherSuccess::Skip(0)), matcher.exec(&parser_context));
  }
}
