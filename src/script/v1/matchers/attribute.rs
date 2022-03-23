#[macro_export]
macro_rules! ScriptAttribute {
  () => {
    $crate::Program!("Attribute";
      $crate::Store!("AttributeStartOffset"; $crate::Pin!()),
      $crate::Matches!("Name"; r"[\w+_]+"),
      $crate::ScriptWS0!(?),
      $crate::Discard!($crate::Equals!("=")),
      $crate::ScriptWS0!(?),
      $crate::AssertNot!($crate::Equals!("'"), "Malformed attribute detected. The proper format for an attribute is: name='value'"),
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
    Loop, Matches, Switch,
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
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 11));
      assert_eq!(token.value(), "test='derp'");
      assert_eq!(token.raw_value(), "test='derp'");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Name");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 4));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 4));
      assert_eq!(first.value(), "test");
      assert_eq!(first.raw_value(), "test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Value");
      assert_eq!(*second.get_value_range(), SourceRange::new(6, 10));
      assert_eq!(*second.get_raw_range(), SourceRange::new(5, 11));
      assert_eq!(second.value(), "derp");
      assert_eq!(second.raw_value(), "'derp'");
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
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), "_test='derp'");
      assert_eq!(token.raw_value(), "_test='derp'");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Name");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 5));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 5));
      assert_eq!(first.value(), "_test");
      assert_eq!(first.raw_value(), "_test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Value");
      assert_eq!(*second.get_value_range(), SourceRange::new(7, 11));
      assert_eq!(*second.get_raw_range(), SourceRange::new(6, 12));
      assert_eq!(second.value(), "derp");
      assert_eq!(second.raw_value(), "'derp'");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Error");
      assert_eq!(*third.get_value_range(), SourceRange::new(0, 5));
      assert_eq!(*third.get_raw_range(), SourceRange::new(0, 5));
      assert_eq!(third.value(), "_test");
      assert_eq!(third.raw_value(), "_test");
      assert_eq!(
        third.get_attribute("__message".to_string()).unwrap(),
        "Attribute names can not start with an underscore"
      );
    } else {
      unreachable!("Test failed!");
    };
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