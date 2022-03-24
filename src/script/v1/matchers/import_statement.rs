#[macro_export]
macro_rules! ScriptImportStatement {
  () => {
    $crate::Program!("ImportStatement";
      $crate::Discard!($crate::Equals!("import")),
      $crate::ScriptWS0!(?),
      $crate::ScriptString!("Path"),
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
    let parser = Parser::new("import '../test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "ImportStatement");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 16));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 16));
      assert_eq!(token.value(), "import '../test'");
      assert_eq!(token.raw_value(), "import '../test'");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Path");
      assert_eq!(*first.get_value_range(), SourceRange::new(8, 15));
      assert_eq!(*first.get_raw_range(), SourceRange::new(7, 16));
      assert_eq!(first.value(), "../test");
      assert_eq!(first.raw_value(), "'../test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing 'derp'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_fails2() {
    let parser = Parser::new("import\n'test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
