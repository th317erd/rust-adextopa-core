#[macro_export]
macro_rules! ScriptCustomMatcher {
  () => {
    $crate::Program!("CustomMatcher";
      $crate::ScriptIdentifier!(),
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("test");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptCustomMatcher!();

    let result = matcher.borrow().exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "CustomMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 4));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 4));
      assert_eq!(token.value(), "test");
      assert_eq!(token.raw_value(), "test");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("<test>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptCustomMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.borrow().exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
