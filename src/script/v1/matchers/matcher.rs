#[macro_export]
macro_rules! ScriptMatcher {
  () => {
    $crate::Switch!("Matcher";
      $crate::ScriptRegexMatcher!(),
      $crate::ScriptEqualsMatcher!(),
      $crate::ScriptSequenceMatcher!(),
      $crate::ScriptCustomMatcher!(),
      $crate::Ref!("SwitchMatcher"),
      $crate::Ref!("ProgramMatcher"),
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
    Loop, ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  fn register_matchers(parser_context: &ParserContextRef) {
    (*parser_context)
      .borrow()
      .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);
  }

  #[test]
  fn it_works1() {
    let parser = Parser::new("='test'%'[',']',''/test/icustom");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = Loop!(ScriptMatcher!());

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(2, 31));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 31));
      assert_eq!(token.get_value(), "test'%'[',']',''/test/icustom");
      assert_eq!(token.get_matched_value(), "='test'%'[',']',''/test/icustom");
      assert_eq!(token.get_children().len(), 4);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "EqualsMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(2, 6));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "='test'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "SequenceMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(9, 17));
      assert_eq!(*second.get_matched_range(), SourceRange::new(7, 18));
      assert_eq!(second.get_value(), "[',']','");
      assert_eq!(second.get_matched_value(), "%'[',']',''");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "RegexMatcher");
      assert_eq!(*third.get_captured_range(), SourceRange::new(18, 25));
      assert_eq!(*third.get_matched_range(), SourceRange::new(18, 25));
      assert_eq!(third.get_value(), "test");
      assert_eq!(third.get_matched_value(), "/test/i");

      let forth = token.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "CustomMatcher");
      assert_eq!(*forth.get_captured_range(), SourceRange::new(25, 31));
      assert_eq!(*forth.get_matched_range(), SourceRange::new(25, 31));
      assert_eq!(forth.get_value(), "custom");
      assert_eq!(forth.get_matched_value(), "custom");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("='test'");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "EqualsMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(2, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "test");
      assert_eq!(token.get_matched_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new(r"/\s+/");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(token.get_value(), r"\s+");
      assert_eq!(token.get_matched_value(), r"/\s+/");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("!");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptMatcher!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
