#[macro_export]
macro_rules! ScriptMatcherName {
  () => {
    $crate::Program!("MatcherName";
      $crate::Discard!($crate::Equals!("?")),
      $crate::ScriptString!("Name"),
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("?'test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptMatcherName!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "MatcherName");
      assert_eq!(*token.get_captured_range(), SourceRange::new(2, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "test");
      assert_eq!(token.get_matched_value(), "?'test'");
      assert_eq!(token.get_children().len(), 1);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptMatcherName!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
