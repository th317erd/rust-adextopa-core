#[macro_export]
macro_rules! ScriptPatternScope {
  () => {
    $crate::Loop!(1..; "PatternScope";
      $crate::ScriptWSN0!(?),
      $crate::Switch!(
        $crate::ScriptComment!(),
        $crate::ScriptPattern!(),
      ),
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure,
    parser::Parser,
    parser_context::{ParserContext, ParserContextRef},
    source_range::SourceRange,
    ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  fn register_matchers(parser_context: &ParserContextRef) {
    (*parser_context)
      .borrow()
      .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);
  }

  #[test]
  fn it_works1() {
    let source = " <='test'>\n\t(</test/i>)\n";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(4, 21));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), "test'>\n\t(</test/i");
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinition");
      assert_eq!(*first.get_captured_range(), SourceRange::new(4, 8));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 10));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "<='test'>");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*second.get_captured_range(), SourceRange::new(14, 21));
      assert_eq!(*second.get_matched_range(), SourceRange::new(12, 23));
      assert_eq!(second.get_value(), "/test/i");
      assert_eq!(second.get_matched_value(), "(</test/i>)");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let source = "\n (<Test>) </\\s+/>(</test/i>)\n";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(4, 27));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 30));
      assert_eq!(token.get_value(), "Test>) </\\s+/>(</test/i");
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*first.get_captured_range(), SourceRange::new(4, 8));
      assert_eq!(*first.get_matched_range(), SourceRange::new(2, 10));
      assert_eq!(first.get_value(), "Test");
      assert_eq!(first.get_matched_value(), "(<Test>)");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinition");
      assert_eq!(*second.get_captured_range(), SourceRange::new(12, 17));
      assert_eq!(*second.get_matched_range(), SourceRange::new(11, 18));
      assert_eq!(second.get_value(), "/\\s+/");
      assert_eq!(second.get_matched_value(), "</\\s+/>");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*third.get_captured_range(), SourceRange::new(20, 27));
      assert_eq!(*third.get_matched_range(), SourceRange::new(18, 29));
      assert_eq!(third.get_value(), "/test/i");
      assert_eq!(third.get_matched_value(), "(</test/i>)");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("<!--[adextopa version='1']-->");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternScope!();

    register_matchers(&parser_context);

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
