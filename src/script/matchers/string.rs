#[macro_export]
macro_rules! ScriptString {
  () => {
    $crate::Sequence!("String"; "'", "'", "\\")
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("'A \\'test\\' string!' after string");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptString!();

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "String");
      assert_eq!(*token.get_value_range(), SourceRange::new(1, 19));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 20));
      assert_eq!(token.value(), "A \\'test\\' string!");
      assert_eq!(token.raw_value(), "'A \\'test\\' string!'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptString!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
