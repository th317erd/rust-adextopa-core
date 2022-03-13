use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::{ParserContext, ParserContextRef};

pub struct DiscardPattern {
  matcher: Box<dyn Matcher>,
}

impl DiscardPattern {
  pub fn new(matcher: Box<dyn Matcher>) -> Self {
    Self { matcher }
  }
}

impl Matcher for DiscardPattern {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match self.matcher.exec(context.clone()) {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          let offset: isize =
            token.borrow().get_raw_range().end as isize - context.borrow().offset.start as isize;
          return Ok(MatcherSuccess::Skip(offset));
        }
        MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Skip(offset)),
        _ => Ok(success),
      },
      Err(failure) => return Err(failure),
    }
  }

  fn get_name(&self) -> &str {
    "Discard"
  }
}

#[macro_export]
macro_rules! Discard {
  ($arg:expr) => {
    $crate::matchers::discard::DiscardPattern::new(std::boxed::Box::new($arg))
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
    let matcher = Discard!(Equals!("Testing"));

    assert_eq!(
      Ok(MatcherSuccess::Skip(7)),
      matcher.exec(parser_context.clone())
    );
  }

  #[test]
  fn it_fails_to_match_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Discard!(Equals!("testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      matcher.exec(parser_context.clone())
    );
  }
}
