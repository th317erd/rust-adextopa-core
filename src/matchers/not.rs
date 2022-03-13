use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::{ParserContext, ParserContextRef};

pub struct NotPattern {
  matcher: Box<dyn Matcher>,
}

impl NotPattern {
  pub fn new(matcher: Box<dyn Matcher>) -> Self {
    Self { matcher }
  }
}

impl Matcher for NotPattern {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match self.matcher.exec(context) {
      Ok(success) => match success {
        // Fail on success
        _ => return Err(MatcherFailure::Fail),
      },
      Err(failure) => match failure {
        // Succeed on fail
        MatcherFailure::Fail => Ok(MatcherSuccess::Skip(0)),
        MatcherFailure::Error(err) => Err(MatcherFailure::Error(err)),
      },
    }
  }

  fn get_name(&self) -> &str {
    "Not"
  }
}

#[macro_export]
macro_rules! Not {
  ($arg:expr) => {
    $crate::matchers::not::NotPattern::new(std::boxed::Box::new($arg))
  };
}

mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Not!(Equals!("Testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      matcher.exec(parser_context.clone())
    );
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Not!(Equals!("testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      matcher.exec(parser_context.clone())
    );
  }
}
