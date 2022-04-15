#[macro_export]
macro_rules! ScriptAttributes {
  () => {
    $crate::Optional!(
      $crate::Loop!(1..; "Attributes";
        $crate::ScriptAttribute!(),
        $crate::ScriptWSN0!(?),
      )
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parse_error::ParseError, parser::Parser,
    parser_context::ParserContext, source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("test='derp' stuff='things'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttributes!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Attributes");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 25));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 26));
      assert_eq!(token.get_value(), "test='derp' stuff='things");
      assert_eq!(token.get_matched_value(), "test='derp' stuff='things'");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Attribute");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 10));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(first.get_value(), "test='derp");
      assert_eq!(first.get_matched_value(), "test='derp'");
      assert_eq!(first.get_children().len(), 2);

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Attribute");
      assert_eq!(*second.get_captured_range(), SourceRange::new(12, 25));
      assert_eq!(*second.get_matched_range(), SourceRange::new(12, 26));
      assert_eq!(second.get_value(), "stuff='things");
      assert_eq!(second.get_matched_value(), "stuff='things'");
      assert_eq!(second.get_children().len(), 2);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_reports_an_error_when_the_name_starts_with_underscore() {
    let parser = Parser::new("_test='derp'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttributes!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Attributes");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), "_test='derp");
      assert_eq!(token.get_matched_value(), "_test='derp'");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Attribute");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(first.get_value(), "_test='derp");
      assert_eq!(first.get_matched_value(), "_test='derp'");
      assert_eq!(first.get_children().len(), 2);

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Error");
      assert_eq!(*second.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*second.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(second.get_value(), "_test");
      assert_eq!(second.get_matched_value(), "_test");
      assert_eq!(second.get_children().len(), 0);
      assert_eq!(
        second.get_attribute("__message"),
        Some(&"Error: @[1:6]: Attribute names can not start with an underscore".to_string())
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_panics_when_the_value_is_not_a_string() {
    let parser = Parser::new("_test=derp");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttributes!();

    let result = ParserContext::tokenize(parser_context, matcher);

    assert_eq!(
      Err(MatcherFailure::Error(
        ParseError::new_with_range(
          "Error: @[1:7-11]: Malformed attribute detected. Attribute value is not single-quoted. The proper format for an attribute is: name='value'",
          SourceRange::new(6, 10)
        )
      )),
      result
    );
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttributes!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
