#[macro_export]
macro_rules! ScriptAttribute {
  () => {
    $crate::Program!("Attribute";
      $crate::Store!("AttributeStartOffset"; $crate::Pin!()),
      $crate::Matches!("Name"; r"[\w+_]+"),
      $crate::ScriptWS0!(?),
      $crate::Discard!($crate::Equals!("=")),
      $crate::ScriptWS0!(?),
      $crate::PanicNot!($crate::Equals!("'"), "Malformed attribute detected. Attribute value is not single-quoted. The proper format for an attribute is: name='value'"),
      $crate::ScriptString!("Value"),
      $crate::Pin!($crate::Fetch!("AttributeStartOffset.range");
        $crate::Assert!(
          $crate::Matches!(r"_[\w+_]+"),
          "Attribute names can not start with an underscore"
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
    let parser = Parser::new("test='derp'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttribute!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Attribute");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 10));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_captured_value(), "test='derp");
      assert_eq!(token.get_matched_value(), "test='derp'");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Name");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
      assert_eq!(first.get_captured_value(), "test");
      assert_eq!(first.get_matched_value(), "test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Value");
      assert_eq!(*second.get_captured_range(), SourceRange::new(6, 10));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 11));
      assert_eq!(second.get_captured_value(), "derp");
      assert_eq!(second.get_matched_value(), "'derp'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_reports_an_error_when_the_name_starts_with_underscore() {
    let parser = Parser::new("_test='derp'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttribute!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Attribute");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_captured_value(), "_test='derp");
      assert_eq!(token.get_matched_value(), "_test='derp'");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Name");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(first.get_captured_value(), "_test");
      assert_eq!(first.get_matched_value(), "_test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Value");
      assert_eq!(*second.get_captured_range(), SourceRange::new(7, 11));
      assert_eq!(*second.get_matched_range(), SourceRange::new(6, 12));
      assert_eq!(second.get_captured_value(), "derp");
      assert_eq!(second.get_matched_value(), "'derp'");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Error");
      assert_eq!(*third.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*third.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(third.get_captured_value(), "_test");
      assert_eq!(third.get_matched_value(), "_test");
      assert_eq!(
        third.get_attribute("__message").unwrap(),
        "Attribute names can not start with an underscore"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_panics_when_the_value_is_not_a_string() {
    let parser = Parser::new("_test=derp");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttribute!();

    let result = ParserContext::tokenize(parser_context, matcher);

    assert_eq!(
      Err(MatcherFailure::Error(
        "Malformed attribute detected. Attribute value is not single-quoted. The proper format for an attribute is: name='value'"
          .to_string()
      )),
      result
    );
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptAttribute!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
