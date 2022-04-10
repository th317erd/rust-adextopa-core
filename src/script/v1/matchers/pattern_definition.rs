#[macro_export]
macro_rules! ScriptPatternDefinition {
  () => {
    $crate::Map!(
      $crate::Program!("PatternDefinition";
        // ? Optional, and ! Not modifiers come first, outside
        // and also inside. The purpose is to inform the parser
        // if they go outside of the loop, or if they go inside
        // the loop. Both at the same time (inside and outside)
        // is valid syntax.

        // Check for both "optional" and "not",
        // which can not both be used at the same time
        $crate::Assert!(
          $crate::Matches!(r"\?!|!\?"),
          "Can not use ? and ! at the same time in this context. Use one or the other, not both."
        ),
        $crate::Optional!($crate::Switch!(
          $crate::Equals!("OuterOptionalModifier"; "?"),
          $crate::Equals!("OuterNotModifier"; "!"),
        )),
        $crate::Discard!($crate::Equals!("<")),
        // Check for both "optional" and "not",
        // which can not both be used at the same time
        $crate::Assert!(
          $crate::Matches!(r"\?!|!\?"),
          "Can not use ? and ! at the same time in this context. Use one or the other, not both."
        ),
        $crate::Optional!($crate::Switch!(
          $crate::Equals!("InnerOptionalModifier"; "?"),
          $crate::Equals!("InnerNotModifier"; "!"),
        )),
        $crate::ScriptWSN0!(?),
        $crate::ScriptMatcher!(),
        $crate::ScriptWSN0!(?),
        $crate::ScriptAttributes!(),
        $crate::Discard!($crate::Equals!(">")),
        $crate::Optional!($crate::ScriptRepeatSpecifier!()),
      ),
      |token| {
        let mut token = token.borrow_mut();

        if let Some(attribute_token) = token.find_child("Attributes") {
          for attribute in attribute_token.borrow().get_children() {
            let _attribute = attribute.borrow();
            let children = _attribute.get_children();

            let name = &children[0];
            let value = &children[1];

            token.set_attribute(name.borrow().get_value(), value.borrow().get_value());
          }
        }

        Ok(())
      }
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
    let parser = Parser::new("<!/test/i>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
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
      assert_eq!(*second.get_captured_range(), SourceRange::new(2, 9));
      assert_eq!(*second.get_matched_range(), SourceRange::new(2, 9));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "/test/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works_with_attributes() {
    let parser = Parser::new("<!/test/i attr1='test' attr2 = 'derp'>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 36));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 38));
      assert_eq!(token.get_value(), "!/test/i attr1='test' attr2 = 'derp");
      assert_eq!(
        token.get_matched_value(),
        "<!/test/i attr1='test' attr2 = 'derp'>"
      );
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "InnerNotModifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 2));
      assert_eq!(first.get_value(), "!");
      assert_eq!(first.get_matched_value(), "!");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RegexMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(2, 9));
      assert_eq!(*second.get_matched_range(), SourceRange::new(2, 9));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "/test/i");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Attributes");
      assert_eq!(*third.get_captured_range(), SourceRange::new(10, 36));
      assert_eq!(*third.get_matched_range(), SourceRange::new(10, 37));
      assert_eq!(third.get_value(), "attr1='test' attr2 = 'derp");
      assert_eq!(third.get_matched_value(), "attr1='test' attr2 = 'derp'");
      assert_eq!(third.get_children().len(), 2);

      let attr1 = third.get_children()[0].borrow();
      assert_eq!(attr1.get_name(), "Attribute");
      assert_eq!(*attr1.get_captured_range(), SourceRange::new(10, 21));
      assert_eq!(*attr1.get_matched_range(), SourceRange::new(10, 22));
      assert_eq!(attr1.get_value(), "attr1='test");
      assert_eq!(attr1.get_matched_value(), "attr1='test'");

      let attr2 = third.get_children()[1].borrow();
      assert_eq!(attr2.get_name(), "Attribute");
      assert_eq!(*attr2.get_captured_range(), SourceRange::new(23, 36));
      assert_eq!(*attr2.get_matched_range(), SourceRange::new(23, 37));
      assert_eq!(attr2.get_value(), "attr2 = 'derp");
      assert_eq!(attr2.get_matched_value(), "attr2 = 'derp'");

      assert_eq!(token.get_attribute("attr1"), Some(&"test".to_string()));
      assert_eq!(token.get_attribute("attr2"), Some(&"derp".to_string()));
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("!<='test'>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 8));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "!<='test");
      assert_eq!(token.get_matched_value(), "!<='test'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "OuterNotModifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 1));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 1));
      assert_eq!(first.get_value(), "!");
      assert_eq!(first.get_matched_value(), "!");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "EqualsMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(4, 8));
      assert_eq!(*second.get_matched_range(), SourceRange::new(2, 9));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new("<?='test'>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 8));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "?='test");
      assert_eq!(token.get_matched_value(), "<?='test'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "InnerOptionalModifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 2));
      assert_eq!(first.get_value(), "?");
      assert_eq!(first.get_matched_value(), "?");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "EqualsMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(4, 8));
      assert_eq!(*second.get_matched_range(), SourceRange::new(2, 9));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works4() {
    let parser = Parser::new("< %'[',']','' >");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(4, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 15));
      assert_eq!(token.get_value(), "[',']','");
      assert_eq!(token.get_matched_value(), "< %'[',']','' >");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "SequenceMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(4, 12));
      assert_eq!(*first.get_matched_range(), SourceRange::new(2, 13));
      assert_eq!(first.get_value(), "[',']','");
      assert_eq!(first.get_matched_value(), "%'[',']',''");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works5() {
    let parser = Parser::new(r"</\s+/>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), r"/\s+/");
      assert_eq!(token.get_matched_value(), r"</\s+/>");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "RegexMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 6));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 6));
      assert_eq!(first.get_value(), r"\s+");
      assert_eq!(first.get_matched_value(), r"/\s+/");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works_with_repeat_specifier1() {
    let parser = Parser::new("<='test'>+");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(3, 10));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "test'>+");
      assert_eq!(token.get_matched_value(), "<='test'>+");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "EqualsMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(3, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 8));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "='test'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RepeatOneOrMore");
      assert_eq!(*second.get_captured_range(), SourceRange::new(9, 10));
      assert_eq!(*second.get_matched_range(), SourceRange::new(9, 10));
      assert_eq!(second.get_value(), "+");
      assert_eq!(second.get_matched_value(), "+");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works_with_repeat_specifier2() {
    let parser = Parser::new("<='test'>*");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(3, 10));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "test'>*");
      assert_eq!(token.get_matched_value(), "<='test'>*");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "EqualsMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(3, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 8));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "='test'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RepeatZeroOrMore");
      assert_eq!(*second.get_captured_range(), SourceRange::new(9, 10));
      assert_eq!(*second.get_matched_range(), SourceRange::new(9, 10));
      assert_eq!(second.get_value(), "*");
      assert_eq!(second.get_matched_value(), "*");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works_with_repeat_specifier3() {
    let parser = Parser::new("<='test'>{2,}");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(3, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 13));
      assert_eq!(token.get_value(), "test'>{2,");
      assert_eq!(token.get_matched_value(), "<='test'>{2,}");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "EqualsMatcher");
      assert_eq!(*first.get_captured_range(), SourceRange::new(3, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 8));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "='test'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RepeatRange");
      assert_eq!(*second.get_captured_range(), SourceRange::new(10, 12));
      assert_eq!(*second.get_matched_range(), SourceRange::new(9, 13));
      assert_eq!(second.get_value(), "2,");
      assert_eq!(second.get_matched_value(), "{2,}");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_on_not_optional1() {
    let parser = Parser::new("<?!='test'>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 9));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_value(), "?!='test");
      assert_eq!(token.get_matched_value(), "<?!='test'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Error");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 3));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 3));
      assert_eq!(first.get_value(), "?!");
      assert_eq!(first.get_matched_value(), "?!");
      assert_eq!(
        first.get_attribute("__message"),
        Some(
          &"Can not use ? and ! at the same time in this context. Use one or the other, not both."
            .to_string()
        )
      );

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "EqualsMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 9));
      assert_eq!(*second.get_matched_range(), SourceRange::new(3, 10));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_on_not_optional2() {
    let parser = Parser::new("!?<='test'>");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "PatternDefinition");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 9));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_value(), "!?<='test");
      assert_eq!(token.get_matched_value(), "!?<='test'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Error");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 2));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 2));
      assert_eq!(first.get_value(), "!?");
      assert_eq!(first.get_matched_value(), "!?");
      assert_eq!(
        first.get_attribute("__message"),
        Some(
          &"Can not use ? and ! at the same time in this context. Use one or the other, not both."
            .to_string()
        )
      );

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "EqualsMatcher");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 9));
      assert_eq!(*second.get_matched_range(), SourceRange::new(3, 10));
      assert_eq!(second.get_value(), "test");
      assert_eq!(second.get_matched_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");

    register_matchers(&parser_context);

    let matcher = ScriptPatternDefinition!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
