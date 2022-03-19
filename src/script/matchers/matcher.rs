#[macro_export]
macro_rules! ScriptMatcher {
  () => {
    $crate::Switch!("Matcher";
      $crate::ScriptRegexMatcher!(),
      $crate::ScriptEqualsMatcher!(),
      $crate::ScriptSequenceMatcher!(),
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Loop,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("='test'%'[',']',''/test/i");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(ScriptMatcher!());

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 25));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 25));
      assert_eq!(token.value(), "='test'%'[',']',''/test/i");
      assert_eq!(token.raw_value(), "='test'%'[',']',''/test/i");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "EqualsMatcher");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(first.value(), "='test'");
      assert_eq!(first.raw_value(), "='test'");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "SequenceMatcher");
      assert_eq!(*second.get_value_range(), SourceRange::new(7, 18));
      assert_eq!(*second.get_raw_range(), SourceRange::new(7, 18));
      assert_eq!(second.value(), "%'[',']',''");
      assert_eq!(second.raw_value(), "%'[',']',''");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "RegexMatcher");
      assert_eq!(*third.get_value_range(), SourceRange::new(18, 25));
      assert_eq!(*third.get_raw_range(), SourceRange::new(18, 25));
      assert_eq!(third.value(), "/test/i");
      assert_eq!(third.raw_value(), "/test/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("='test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "EqualsMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "='test'");
      assert_eq!(token.raw_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
