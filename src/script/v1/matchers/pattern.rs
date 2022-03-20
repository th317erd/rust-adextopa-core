#[macro_export]
macro_rules! ScriptPattern {
  () => {
    $crate::Switch!("Pattern";
      $crate::Program!("PatternDefinitionCaptured";
        $crate::Discard!($crate::Equals!("(")),
        $crate::ScriptWSN0!(?),
        $crate::Optional!($crate::ScriptMatcherName!()),
        $crate::ScriptWSN0!(?),
        $crate::ScriptPatternDefinition!(),
        $crate::ScriptWSN0!(?),
        $crate::Discard!($crate::Equals!(")")),
      ),
      $crate::ScriptPatternDefinition!(),
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
    let parser = Parser::new("<!/test/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 10));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 10));
      assert_eq!(token.value(), "<!/test/i>");
      assert_eq!(token.raw_value(), "<!/test/i>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "NotModifier");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 2));
      assert_eq!(first.value(), "!");
      assert_eq!(first.raw_value(), "!");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RegexMatcher");
      assert_eq!(*second.get_value_range(), SourceRange::new(2, 9));
      assert_eq!(*second.get_raw_range(), SourceRange::new(2, 9));
      assert_eq!(second.value(), "/test/i");
      assert_eq!(second.raw_value(), "/test/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("(<!/test/i>)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), "(<!/test/i>)");
      assert_eq!(token.raw_value(), "(<!/test/i>)");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinition");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 11));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 11));
      assert_eq!(first.value(), "<!/test/i>");
      assert_eq!(first.raw_value(), "<!/test/i>");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new("(\n\t?'name'\n\t<!/test/i>\n)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 24));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 24));
      assert_eq!(token.value(), "(\n\t?'name'\n\t<!/test/i>\n)");
      assert_eq!(token.raw_value(), "(\n\t?'name'\n\t<!/test/i>\n)");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "MatcherName");
      assert_eq!(*first.get_value_range(), SourceRange::new(3, 10));
      assert_eq!(*first.get_raw_range(), SourceRange::new(3, 10));
      assert_eq!(first.value(), "?'name'");
      assert_eq!(first.raw_value(), "?'name'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinition");
      assert_eq!(*second.get_value_range(), SourceRange::new(12, 22));
      assert_eq!(*second.get_raw_range(), SourceRange::new(12, 22));
      assert_eq!(second.value(), "<!/test/i>");
      assert_eq!(second.raw_value(), "<!/test/i>");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("(test)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}