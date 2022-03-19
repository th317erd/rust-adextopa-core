use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef};

pub struct DiscardPattern {
  matcher: Box<dyn Matcher>,
}

impl DiscardPattern {
  pub fn new(matcher: Box<dyn Matcher>) -> Self {
    Self { matcher }
  }
}

fn collect_errors<'a, 'b>(error_token: TokenRef<'a>, walk_token: TokenRef<'a>) {
  if walk_token.borrow().get_name() == "Error" {
    error_token
      .borrow_mut()
      .get_children_mut()
      .push(walk_token.clone());
  }

  {
    let walk_token = walk_token.borrow();
    let children = walk_token.get_children();
    if children.len() > 0 {
      for child in children {
        collect_errors(error_token.clone(), child.clone());
      }
    }
  }
}

impl Matcher for DiscardPattern {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = std::rc::Rc::new(std::cell::RefCell::new(context.borrow().clone()));
    let start_offset = context.borrow().offset.start;

    match self.matcher.exec(sub_context.clone()) {
      Ok(success) => match success {
        MatcherSuccess::Token(token) => {
          let end_offset = token.borrow().get_raw_range().end;
          let offset: isize = end_offset as isize - start_offset as isize;

          // Check to see if any errors are in the result
          // If there are, continue to proxy them upstream
          let error_token = StandardToken::new(
            &context.borrow().parser,
            "Error",
            SourceRange::new(start_offset, end_offset),
          );
          collect_errors(error_token.clone(), token.clone());

          if error_token.borrow().get_children().len() > 0 {
            return Ok(MatcherSuccess::Token(error_token));
          }

          return Ok(MatcherSuccess::Skip(offset));
        }
        MatcherSuccess::Skip(offset) => Ok(MatcherSuccess::Skip(offset)),
        _ => Ok(success),
      },
      Err(failure) => {
        return Err(failure);
      }
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

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals, Error, Program,
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
  fn it_properly_returns_error_tokens() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Discard!(Program!(Equals!("Testing"), Error!("This is an error")));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");
      assert_eq!(token.get_children().len(), 1);
      assert_eq!(token.get_attribute("__message".to_string()), None);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Error");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(first.value(), "Testing");
      assert_eq!(first.raw_value(), "Testing");
      assert_eq!(first.get_children().len(), 0);
      assert_eq!(
        first.get_attribute("__message".to_string()),
        Some(&"This is an error".to_string())
      );
    } else {
      unreachable!("Test failed!");
    };
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
