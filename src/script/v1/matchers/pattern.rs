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
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::{ParserContext, ParserContextRef},
    source_range::SourceRange,
    ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  fn register_matchers(parser_context: &ParserContextRef) {
    (*parser_context)
      .borrow()
      .register_matchers(None, vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);
  }

  #[test]
  fn it_works1() {
    let parser = Parser::new("<!/test/i>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPattern!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 9));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "!/test/i");
      assert_eq!(token.get_matched_value(), "<!/test/i>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "InnerNotModifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 2));
      assert_eq!(first.get_value(), "!");
      assert_eq!(first.get_matched_value(), "!");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RegexMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(3, 9));
      assert_eq!(*second.get_matched_range(), SourceRange::new(2, 9));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "/test/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("(<!/test/i>)");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPattern!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*token.get_captured_range(), SourceRange::new(2, 10));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), "!/test/i");
      assert_eq!(token.get_matched_value(), "(<!/test/i>)");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinition");
      assert_eq!(*first.get_captured_range(), SourceRange::new(2, 10));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 11));
      assert_eq!(first.get_value(), "!/test/i");
      assert_eq!(first.get_matched_value(), "<!/test/i>");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new("(\n\t?'name'\n\t<!/test/i>\n)");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPattern!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*token.get_captured_range(), SourceRange::new(5, 21));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), "name'\n\t<!/test/i");
      assert_eq!(token.get_matched_value(), "(\n\t?'name'\n\t<!/test/i>\n)");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "MatcherName");
      assert_eq!(*first.get_captured_range(), SourceRange::new(5, 9));
      assert_eq!(*first.get_matched_range(), SourceRange::new(3, 10));
      assert_eq!(first.get_value(), "name");
      assert_eq!(first.get_matched_value(), "?'name'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinition");
      assert_eq!(*second.get_captured_range(), SourceRange::new(13, 21));
      assert_eq!(*second.get_matched_range(), SourceRange::new(12, 22));
      assert_eq!(second.get_value(), "!/test/i");
      assert_eq!(second.get_matched_value(), "<!/test/i>");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("(test)");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPattern!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
