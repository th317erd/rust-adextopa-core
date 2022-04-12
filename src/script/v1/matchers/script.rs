#[macro_export]
macro_rules! Script {
  () => {
    $crate::Map!(
      $crate::Program!("Script";
        $crate::Optional!(
          $crate::Loop!("PreHead";
            $crate::ScriptWSN0!(?),
            $crate::ScriptComment!(),
          )
        ),
        $crate::ScriptWSN0!(?),
        $crate::Optional!($crate::ScriptAdextopaScope!()),
        $crate::Optional!($crate::ScriptPatternScope!()),
      ),
      |token| {
        let token = token.borrow();
        let adextopa_scope = token.find_child("AdextopaScope");
        let pattern_scope = token.find_child("PatternScope");

        if adextopa_scope.is_none() && pattern_scope.is_none() {
          return Err("Nothing to parse. No scopes found.".to_string());
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
    let source = " <='test'>\n\t(</test/i>)\n";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Script!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();

      assert_eq!(token.get_name(), "Script");
      assert_eq!(*token.get_captured_range(), SourceRange::new(4, 21));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), "test'>\n\t(</test/i");
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternScope");
      assert_eq!(*first.get_captured_range(), SourceRange::new(4, 21));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 24));
      assert_eq!(first.get_value(), "test'>\n\t(</test/i");
      assert_eq!(first.get_matched_value(), &source[1..]);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let source = " \n\n<!--[adextopa version='1']--> <='test'>\n\t(</test/i>)\n";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Script!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Script");
      assert_eq!(*token.get_captured_range(), SourceRange::new(17, 53));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 56));
      assert_eq!(token.get_value(), "version='1']--> <='test'>\n\t(</test/i");
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "AdextopaScope");
      assert_eq!(*first.get_captured_range(), SourceRange::new(17, 27));
      assert_eq!(*first.get_matched_range(), SourceRange::new(3, 32));
      assert_eq!(first.get_value(), "version='1");
      assert_eq!(first.get_matched_value(), "<!--[adextopa version='1']-->");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternScope");
      assert_eq!(*second.get_captured_range(), SourceRange::new(36, 53));
      assert_eq!(*second.get_matched_range(), SourceRange::new(32, 56));
      assert_eq!(second.get_value(), "test'>\n\t(</test/i");
      assert_eq!(second.get_matched_value(), " <='test'>\n\t(</test/i>)\n");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("derp");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Script!();

    register_matchers(&parser_context);

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_fails_when_empty() {
    let source = "# there is nothing here but a comment";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Script!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 37));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 37));
      assert_eq!(token.get_value(), source);
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 0);

      assert_eq!(
        token.get_attribute("__message"),
        Some(&"Nothing to parse. No scopes found.".to_string())
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
