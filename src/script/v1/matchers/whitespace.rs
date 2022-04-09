#[macro_export]
macro_rules! ScriptWS0 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[^\S\n\r]*"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[^\S\n\r]*")
  };
}

#[macro_export]
macro_rules! ScriptWS1 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"[^\S\n\r]+"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"[^\S\n\r]+")
  };
}

#[macro_export]
macro_rules! ScriptWSN0 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"\s*"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"\s*")
  };
}

#[macro_export]
macro_rules! ScriptWSN1 {
  (?) => {
    $crate::Discard!($crate::Matches!("Whitespace"; r"\s+"))
  };

  () => {
    $crate::Matches!("Whitespace"; r"\s+")
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
  fn it_matches_against_zero_or_more() {
    let parser = Parser::new("");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      matcher.borrow().exec(
        matcher.clone(),
        parser_context.clone(),
        parser_context.borrow().scope.clone(),
      )
    );
  }

  #[test]
  fn it_matches_against_one_or_more() {
    let parser = Parser::new("  ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Whitespace");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 2));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 2));
      assert_eq!(token.get_value(), "  ");
      assert_eq!(token.get_matched_value(), "  ");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_will_not_match_against_new_lines() {
    let parser = Parser::new("  \n \t\r\n");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS1!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Whitespace");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 2));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 2));
      assert_eq!(token.get_value(), "  ");
      assert_eq!(token.get_matched_value(), "  ");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_will_match_against_newlines() {
    let parser = Parser::new("\r\n\r  \n");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWSN1!(?);

    assert_eq!(
      Ok(MatcherSuccess::Skip(6)),
      matcher.borrow().exec(
        matcher.clone(),
        parser_context.clone(),
        parser_context.borrow().scope.clone(),
      )
    );
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptWS0!();

    assert_eq!(
      Ok(MatcherSuccess::Skip(0)),
      matcher.borrow().exec(
        matcher.clone(),
        parser_context.clone(),
        parser_context.borrow().scope.clone(),
      )
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
