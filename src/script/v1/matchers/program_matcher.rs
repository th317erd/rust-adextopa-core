#[macro_export]
macro_rules! ScriptProgramMatcher {
  () => {
    $crate::Program!("ProgramMatcher";
      $crate::Discard!($crate::Equals!("{")),
      $crate::Flatten!(
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

    let result = matcher.borrow().exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "ProgramMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 26));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 26));
      assert_eq!(token.value(), "{\n\t<='test'>\n\t(</test/i>)}");
      assert_eq!(token.raw_value(), "{\n\t<='test'>\n\t(</test/i>)}");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "PatternDefinition");
      assert_eq!(*first.get_value_range(), SourceRange::new(3, 12));
      assert_eq!(*first.get_raw_range(), SourceRange::new(3, 12));
      assert_eq!(first.value(), "<='test'>");
      assert_eq!(first.raw_value(), "<='test'>");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinitionCaptured");
      assert_eq!(*second.get_value_range(), SourceRange::new(14, 25));
      assert_eq!(*second.get_raw_range(), SourceRange::new(14, 25));
      assert_eq!(second.value(), "(</test/i>)");
      assert_eq!(second.raw_value(), "(</test/i>)");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("<test>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptProgramMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.borrow().exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}