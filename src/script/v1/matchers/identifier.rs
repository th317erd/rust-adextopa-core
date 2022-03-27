#[macro_export]
macro_rules! ScriptIdentifier {
  ($name:expr) => {
    $crate::Matches!($name; r"[a-zA-Z$_][a-zA-Z0-9$_]*")
  };

  () => {
    $crate::Matches!("Identifier"; r"[a-zA-Z$_][a-zA-Z0-9$_]*")
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
    let parser = Parser::new("_Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptIdentifier!();

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Identifier");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 8));
      assert_eq!(token.value(), "_Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("$Test_ing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptIdentifier!();

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Identifier");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 9));
      assert_eq!(token.value(), "$Test_ing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("0Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptIdentifier!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
