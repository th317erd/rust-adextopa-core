use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::StandardToken;

pub struct ErrorPattern<'a> {
  message: &'a str,
}

impl<'a> ErrorPattern<'a> {
  pub fn new(message: &'a str) -> Self {
    Self { message }
  }
}

impl<'a> Matcher for ErrorPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let value_range = SourceRange::new(usize::MAX, usize::MAX);
    let token = StandardToken::new(&context.borrow().parser, "Error", value_range);

    {
      let mut token = token.borrow_mut();
      token.set_attribute("__message".to_string(), self.message.to_string());
      token.set_attribute("__is_error".to_string(), "true".to_string());
    }

    Ok(MatcherSuccess::Token(token))
  }

  fn get_name(&self) -> &str {
    "Error"
  }
}

#[macro_export]
macro_rules! Error {
  ($message:expr) => {
    $crate::matchers::error::ErrorPattern::new($message)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Discard, Matches, Program,
  };

  #[test]
  fn it_can_record_an_error() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Matches!(r"\w+"),
      Error!("There was an error!"),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+")
    );

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), "Testing 1234");
      assert_eq!(token.raw_value(), "Testing 1234");
      assert_eq!(token.get_children().len(), 3);

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Error");
      assert_eq!(*second.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*second.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(second.value(), "Testing");
      assert_eq!(second.raw_value(), "Testing");
      assert_eq!(
        second.get_attribute("__message".to_string()).unwrap(),
        "There was an error!"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
