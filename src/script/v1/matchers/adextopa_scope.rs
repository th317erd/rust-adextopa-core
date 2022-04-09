#[macro_export]
macro_rules! ScriptAdextopaScope {
  () => {
    $crate::Map!(
      $crate::Program!("AdextopaScope";
        $crate::Discard!($crate::Matches!(r"<!--\[adextopa")),
        $crate::ScriptWS0!(?),
        $crate::ScriptAttributes!(),
        $crate::Discard!($crate::Equals!("]")),
        $crate::Optional!(
          $crate::Loop!("Scope";
            $crate::ScriptWSN0!(?),
            $crate::Switch!(
              $crate::ScriptComment!(),
              $crate::ScriptImportStatement!(),
              $crate::ScriptAssignmentExpression!(),
              $crate::Discard!(
                $crate::Program!(
                  $crate::Pin!($crate::Equals!("-->")),
                  $crate::Break!(),
                )
              )
            ),
          )
        ),
        $crate::Discard!($crate::Equals!("-->")),
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

        let version = token.get_attribute("version");
        if version.is_none() {
          return Some("Adextopa scope must have a 'version' attribute".to_string());
        }

        let version = version.unwrap().as_str();
        if let Err(_) = version.parse::<usize>() {
          return Some("Adextopa 'version' attribute must be a valid integer number".to_string());
        }

        None
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
    let source =
      "<!--[adextopa version='1']\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment\n-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AdextopaScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(14, 95));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 99));
      assert_eq!(
        token.get_value(),
        "version='1']\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment"
      );
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 2);

      let attributes = token.get_children()[0].borrow();
      assert_eq!(attributes.get_name(), "Attributes");
      assert_eq!(*attributes.get_captured_range(), SourceRange::new(14, 24));
      assert_eq!(*attributes.get_matched_range(), SourceRange::new(14, 25));
      assert_eq!(attributes.get_value(), "version='1");
      assert_eq!(attributes.get_matched_value(), "version='1'");
      assert_eq!(attributes.get_children().len(), 1);

      let scope = token.get_children()[1].borrow();
      assert_eq!(scope.get_name(), "Scope");
      assert_eq!(*scope.get_captured_range(), SourceRange::new(28, 95));
      assert_eq!(*scope.get_matched_range(), SourceRange::new(26, 96));
      assert_eq!(
        scope.get_value(),
        "# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment"
      );
      assert_eq!(
        scope.get_matched_value(),
        "\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment\n"
      );
      assert_eq!(scope.get_children().len(), 4);

      let first = scope.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Comment");
      assert_eq!(*first.get_captured_range(), SourceRange::new(28, 49));
      assert_eq!(*first.get_matched_range(), SourceRange::new(28, 49));
      assert_eq!(first.get_value(), "# Just a test comment");
      assert_eq!(first.get_matched_value(), "# Just a test comment");

      let second = scope.get_children()[1].borrow();
      assert_eq!(second.get_name(), "AssignmentExpression");
      assert_eq!(*second.get_captured_range(), SourceRange::new(51, 65));
      assert_eq!(*second.get_matched_range(), SourceRange::new(51, 67));
      assert_eq!(second.get_value(), "test = <='derp");
      assert_eq!(second.get_matched_value(), "test = <='derp'>");

      let third = scope.get_children()[2].borrow();
      assert_eq!(third.get_name(), "AssignmentExpression");
      assert_eq!(*third.get_captured_range(), SourceRange::new(69, 79));
      assert_eq!(*third.get_matched_range(), SourceRange::new(69, 79));
      assert_eq!(third.get_value(), "test2=test");
      assert_eq!(third.get_matched_value(), "test2=test");

      let forth = scope.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "Comment");
      assert_eq!(*forth.get_captured_range(), SourceRange::new(79, 95));
      assert_eq!(*forth.get_matched_range(), SourceRange::new(79, 95));
      assert_eq!(forth.get_value(), "#another comment");
      assert_eq!(forth.get_matched_value(), "#another comment");

      assert_eq!(token.get_attribute("version"), Some(&"1".to_string()));
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let source = "<!--[adextopa version='1' name='Test']-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AdextopaScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(14, 36));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 41));
      assert_eq!(token.get_value(), "version='1' name='Test");
      assert_eq!(
        token.get_matched_value(),
        "<!--[adextopa version='1' name='Test']-->"
      );
      assert_eq!(token.get_children().len(), 1);

      let attributes_token = token.get_children()[0].borrow();
      assert_eq!(attributes_token.get_name(), "Attributes");
      assert_eq!(
        *attributes_token.get_captured_range(),
        SourceRange::new(14, 36)
      );
      assert_eq!(
        *attributes_token.get_matched_range(),
        SourceRange::new(14, 37)
      );
      assert_eq!(attributes_token.get_value(), "version='1' name='Test");
      assert_eq!(
        attributes_token.get_matched_value(),
        "version='1' name='Test'"
      );

      assert_eq!(token.get_attribute("version"), Some(&"1".to_string()));
      assert_eq!(token.get_attribute("name"), Some(&"Test".to_string()));
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_fail_without_a_version_attribute() {
    let source = "<!--[adextopa name='Test']-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 29));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 29));
      assert_eq!(token.get_value(), "<!--[adextopa name='Test']-->");
      assert_eq!(token.get_matched_value(), "<!--[adextopa name='Test']-->");
      assert_eq!(token.get_children().len(), 0);

      assert_eq!(
        token.get_attribute("__message"),
        Some(&"Adextopa scope must have a 'version' attribute".to_string())
      );

      assert_eq!(token.flags_enabled(crate::token::IS_ERROR), true);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_fail_if_unable_to_parse_version_attribute1() {
    let source = "<!--[adextopa version='1.2']-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 31));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 31));
      assert_eq!(token.get_value(), source);
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 0);

      assert_eq!(
        token.get_attribute("__message"),
        Some(&"Adextopa 'version' attribute must be a valid integer number".to_string())
      );

      assert_eq!(token.flags_enabled(crate::token::IS_ERROR), true);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_fail_if_unable_to_parse_version_attribute2() {
    let source = "<!--[adextopa version='']-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 28));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 28));
      assert_eq!(token.get_value(), source);
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 0);

      assert_eq!(
        token.get_attribute("__message"),
        Some(&"Adextopa 'version' attribute must be a valid integer number".to_string())
      );

      assert_eq!(token.flags_enabled(crate::token::IS_ERROR), true);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_fail_if_unable_to_parse_version_attribute3() {
    let source = "<!--[adextopa version='derp']-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Error");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 32));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 32));
      assert_eq!(token.get_value(), source);
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 0);

      assert_eq!(
        token.get_attribute("__message"),
        Some(&"Adextopa 'version' attribute must be a valid integer number".to_string())
      );

      assert_eq!(token.flags_enabled(crate::token::IS_ERROR), true);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing = ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
