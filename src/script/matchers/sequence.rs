#[macro_export]
macro_rules! ScriptSequenceMatcher {
  () => {
    $crate::Program!(
      "SequenceMatcher";
      $crate::Discard!($crate::Matches!(r"\[\s*")),
      $crate::ScriptString!("StartPattern"),
      $crate::Discard!($crate::Matches!(r"\s*;\s*")),
      $crate::ScriptString!("EndPattern"),
      $crate::Discard!($crate::Matches!(r"\s*;\s*")),
      $crate::ScriptString!("EscapePattern"),
      $crate::Discard!($crate::Matches!(r"\s*;?\s*\]")),
    )
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
    let parser = Parser::new(r"['{';'}';'']");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptSequenceMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "SequenceMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), r"['{';'}';'']");
      assert_eq!(token.raw_value(), r"['{';'}';'']");
      assert_eq!(token.get_children().len(), 3);

      let start = token.get_children()[0].borrow();
      assert_eq!(start.get_name(), "StartPattern");
      assert_eq!(*start.get_value_range(), SourceRange::new(2, 3));
      assert_eq!(*start.get_raw_range(), SourceRange::new(1, 4));
      assert_eq!(start.value(), r"{");
      assert_eq!(start.raw_value(), r"'{'");

      let end = token.get_children()[1].borrow();
      assert_eq!(end.get_name(), "EndPattern");
      assert_eq!(*end.get_value_range(), SourceRange::new(6, 7));
      assert_eq!(*end.get_raw_range(), SourceRange::new(5, 8));
      assert_eq!(end.value(), r"}");
      assert_eq!(end.raw_value(), r"'}'");

      let escape = token.get_children()[2].borrow();
      assert_eq!(escape.get_name(), "EscapePattern");
      assert_eq!(*escape.get_value_range(), SourceRange::new(10, 10));
      assert_eq!(*escape.get_raw_range(), SourceRange::new(9, 11));
      assert_eq!(escape.value(), r"");
      assert_eq!(escape.raw_value(), r"''");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"['{'; '}' ; '\\' ; ]");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptSequenceMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "SequenceMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 20));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 20));
      assert_eq!(token.value(), r"['{'; '}' ; '\\' ; ]");
      assert_eq!(token.raw_value(), r"['{'; '}' ; '\\' ; ]");
      assert_eq!(token.get_children().len(), 3);

      let start = token.get_children()[0].borrow();
      assert_eq!(start.get_name(), "StartPattern");
      assert_eq!(*start.get_value_range(), SourceRange::new(2, 3));
      assert_eq!(*start.get_raw_range(), SourceRange::new(1, 4));
      assert_eq!(start.value(), r"{");
      assert_eq!(start.raw_value(), r"'{'");

      let end = token.get_children()[1].borrow();
      assert_eq!(end.get_name(), "EndPattern");
      assert_eq!(*end.get_value_range(), SourceRange::new(7, 8));
      assert_eq!(*end.get_raw_range(), SourceRange::new(6, 9));
      assert_eq!(end.value(), r"}");
      assert_eq!(end.raw_value(), r"'}'");

      let escape = token.get_children()[2].borrow();
      assert_eq!(escape.get_name(), "EscapePattern");
      assert_eq!(*escape.get_value_range(), SourceRange::new(13, 15));
      assert_eq!(*escape.get_raw_range(), SourceRange::new(12, 16));
      assert_eq!(escape.value(), r"\");
      assert_eq!(escape.raw_value(), r"'\\'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptSequenceMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
