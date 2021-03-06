#[macro_export]
macro_rules! ScriptAssignmentExpression {
  () => {
    $crate::Program!("AssignmentExpression";
      $crate::ScriptIdentifier!(),
      $crate::ScriptWSN0!(?),
      $crate::Discard!($crate::Equals!("=")),
      $crate::ScriptWSN0!(?),
      $crate::Switch!(
        $crate::ScriptIdentifier!(),
        $crate::ScriptPatternDefinition!(),
      )
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure},
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
    let parser = Parser::new("test = <='derp'>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAssignmentExpression!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AssignmentExpression");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 14));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 16));
      assert_eq!(token.get_value(), "test = <='derp");
      assert_eq!(token.get_matched_value(), "test = <='derp'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Identifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "PatternDefinition");
      assert_eq!(*second.get_captured_range(), SourceRange::new(10, 14));
      assert_eq!(*second.get_matched_range(), SourceRange::new(7, 16));
      assert_eq!(second.get_value(), "derp");
      assert_eq!(second.get_matched_value(), "<='derp'>");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("test=stuff");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAssignmentExpression!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "AssignmentExpression");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 10));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 10));
      assert_eq!(token.get_value(), "test=stuff");
      assert_eq!(token.get_matched_value(), "test=stuff");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Identifier");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
      assert_eq!(first.get_value(), "test");
      assert_eq!(first.get_matched_value(), "test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Identifier");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 10));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 10));
      assert_eq!(second.get_value(), "stuff");
      assert_eq!(second.get_matched_value(), "stuff");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing = ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAssignmentExpression!();

    register_matchers(&parser_context);

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
