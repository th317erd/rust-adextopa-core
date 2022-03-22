#[macro_export]
macro_rules! ScriptWS0 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[\s\t]*"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[\s\t]*")
  };
}

#[macro_export]
macro_rules! ScriptWS1 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[\s\t]+"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[\s\t]+")
  };
}

#[macro_export]
macro_rules! ScriptWSN0 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[\s\t\r\n]*"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[\s\t\r\n]*")
  };
}

#[macro_export]
macro_rules! ScriptWSN1 {
  (!) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[\s\t\r\n]+"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[\s\t\r\n]+")
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
    let parser = Parser::new("");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    let result = ParserContext::tokenize(parser_context, matcher);
    assert_eq!(Ok(MatcherSuccess::Skip(0)), result);
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("  ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Whitespace");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 2));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 2));
      assert_eq!(token.value(), "  ");
      assert_eq!(token.raw_value(), "  ");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new("  \n \t\r\n");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWSN1!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Whitespace");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "  \n \t\r\n");
      assert_eq!(token.raw_value(), "  \n \t\r\n");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_fails2() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS1!();

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
