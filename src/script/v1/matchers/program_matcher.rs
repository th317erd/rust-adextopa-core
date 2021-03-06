#[macro_export]
macro_rules! ScriptProgramMatcher {
  () => {
    $crate::Program!("ProgramMatcher";
      $crate::Discard!($crate::Equals!("{")),
      $crate::ProxyChildren!(
        $crate::Loop!(
          $crate::ScriptWSN0!(?),
          $crate::ScriptPattern!(),
          $crate::ScriptWSN0!(?),
          $crate::Discard!(
            $crate::Optional!(
              $crate::Program!(
                $crate::Equals!("}"),
                $crate::Break!(),
              )
            )
          )
        )
      )
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
    let parser = Parser::new("{\n\t<='test'>\n\t(</test/i>)}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptProgramMatcher!();

    let result = matcher.borrow().exec(
      matcher.clone(),
      parser_context.clone(),
      parser_context.borrow().scope.clone(),
    );

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "ProgramMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(6, 23));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 26));
      assert_eq!(token.get_value(), "test'>\n\t(</test/i");
      assert_eq!(token.get_matched_value(), "{\n\t<='test'>\n\t(</test/i>)}");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinition");
      assert_eq!(*first.get_captured_range(), SourceRange::new(6, 10));
      assert_eq!(*first.get_matched_range(), SourceRange::new(3, 12));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "<='test'>");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*second.get_captured_range(), SourceRange::new(16, 23));
      assert_eq!(*second.get_matched_range(), SourceRange::new(14, 25));
      assert_eq!(second.get_value(), "/test/i");
      assert_eq!(second.get_matched_value(), "(</test/i>)");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("<test>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptProgramMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.borrow().exec(
      matcher.clone(),
      parser_context.clone(),
      parser_context.borrow().scope.clone(),
    ) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
