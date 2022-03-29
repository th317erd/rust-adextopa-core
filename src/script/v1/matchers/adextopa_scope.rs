#[macro_export]
macro_rules! ScriptAdextopaScope {
  () => {
    $crate::Program!("AdextopaScope";
      $crate::Discard!($crate::Matches!(r"<!--\[adextopa:v")),
      $crate::Switch!(
        $crate::Matches!("Version"; r"\d+"),
        $crate::Fatal!("You must specify an ADEXTOPA version in your `adextopa:` scope: i.e. `<!--[adextopa:v{version}`"),
      ),
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
    let source =
      "<!--[adextopa:v1]\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment\n-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AdextopaScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(15, 86));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 90));
      assert_eq!(
        token.get_captured_value(),
        "1]\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment"
      );
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 2);

      let version = token.get_children()[0].borrow();
      assert_eq!(version.get_name(), "Version");
      assert_eq!(*version.get_captured_range(), SourceRange::new(15, 16));
      assert_eq!(*version.get_matched_range(), SourceRange::new(15, 16));
      assert_eq!(version.get_captured_value(), "1");
      assert_eq!(version.get_matched_value(), "1");

      let scope = token.get_children()[1].borrow();
      assert_eq!(scope.get_name(), "Scope");
      assert_eq!(*scope.get_captured_range(), SourceRange::new(19, 86));
      assert_eq!(*scope.get_matched_range(), SourceRange::new(17, 87));
      assert_eq!(
        scope.get_captured_value(),
        "# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment"
      );
      assert_eq!(
        scope.get_matched_value(),
        "\n\t# Just a test comment\n\ttest = <='derp'>\n\ttest2=test#another comment\n"
      );
      assert_eq!(scope.get_children().len(), 4);

      let first = scope.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Comment");
      assert_eq!(*first.get_captured_range(), SourceRange::new(19, 40));
      assert_eq!(*first.get_matched_range(), SourceRange::new(19, 40));
      assert_eq!(first.get_captured_value(), "# Just a test comment");
      assert_eq!(first.get_matched_value(), "# Just a test comment");

      let second = scope.get_children()[1].borrow();
      assert_eq!(second.get_name(), "AssignmentExpression");
      assert_eq!(*second.get_captured_range(), SourceRange::new(42, 56));
      assert_eq!(*second.get_matched_range(), SourceRange::new(42, 58));
      assert_eq!(second.get_captured_value(), "test = <='derp");
      assert_eq!(second.get_matched_value(), "test = <='derp'>");

      let third = scope.get_children()[2].borrow();
      assert_eq!(third.get_name(), "AssignmentExpression");
      assert_eq!(*third.get_captured_range(), SourceRange::new(60, 70));
      assert_eq!(*third.get_matched_range(), SourceRange::new(60, 70));
      assert_eq!(third.get_captured_value(), "test2=test");
      assert_eq!(third.get_matched_value(), "test2=test");

      let forth = scope.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "Comment");
      assert_eq!(*forth.get_captured_range(), SourceRange::new(70, 86));
      assert_eq!(*forth.get_matched_range(), SourceRange::new(70, 86));
      assert_eq!(forth.get_captured_value(), "#another comment");
      assert_eq!(forth.get_matched_value(), "#another comment");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let source = "<!--[adextopa:v1]-->";
    let parser = Parser::new(source);
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAdextopaScope!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AdextopaScope");
      assert_eq!(*token.get_captured_range(), SourceRange::new(15, 16));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 20));
      assert_eq!(token.get_captured_value(), "1");
      assert_eq!(token.get_matched_value(), source);
      assert_eq!(token.get_children().len(), 1);

      let version = token.get_children()[0].borrow();
      assert_eq!(version.get_name(), "Version");
      assert_eq!(*version.get_captured_range(), SourceRange::new(15, 16));
      assert_eq!(*version.get_matched_range(), SourceRange::new(15, 16));
      assert_eq!(version.get_captured_value(), "1");
      assert_eq!(version.get_matched_value(), "1");
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
