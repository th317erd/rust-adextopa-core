#[macro_export]
macro_rules! ScriptString {
  ($name:expr) => {
    $crate::Sequence!($name; "'", "'", "\\")
  };

  () => {
    $crate::Sequence!("String"; "'", "'", "\\")
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
    let parser = Parser::new("'A \\'test\\' string!' after string");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptString!();

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "String");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 19));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 20));
      assert_eq!(token.value(), "A 'test' string!");
      assert_eq!(token.raw_value(), "'A \\'test\\' string!'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptString!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
