#[macro_export]
macro_rules! ScriptPattern {
  () => {
    $crate::Program!("Pattern";
      $crate::Discard!($crate::Equals!("<")),
      // Check for both "optional" and "not",
      // which can not both be used at the same time
      $crate::Debug!($crate::Assert!(
        $crate::Matches!(r"\?!|!\?"),
        "Can not use ? and ! at the same time in this context. Use one or the other, not both."
      )),
      $crate::Debug!(),
      $crate::Optional!($crate::Switch!(
        $crate::Equals!("OptionalModifier"; "?"),
        $crate::Equals!("NotModifier"; "!"),
      )),
      $crate::Discard!($crate::Matches!(r"\s*")),
      $crate::Optional!($crate::ScriptMatcherName!()),
      $crate::Discard!($crate::Matches!(r"\s*")),
      $crate::Debug!($crate::ScriptMatcher!()),
      // $crate::Discard!($crate::Matches!(r"\s*")),
      // $crate::Discard!($crate::Equals!(">")),
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
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("<!/test/i>");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Pattern");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 10));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 10));
      assert_eq!(token.value(), "<!/test/i>");
      assert_eq!(token.raw_value(), "<!/test/i>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "NotModifier");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 2));
      assert_eq!(first.value(), "!");
      assert_eq!(first.raw_value(), "!");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "RegexMatcher");
      assert_eq!(*second.get_value_range(), SourceRange::new(2, 9));
      assert_eq!(*second.get_raw_range(), SourceRange::new(2, 9));
      assert_eq!(second.value(), "/test/i");
      assert_eq!(second.raw_value(), "/test/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("<?='test'>");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Pattern");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 10));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 10));
      assert_eq!(token.value(), "<?='test'>");
      assert_eq!(token.raw_value(), "<?='test'>");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "OptionalModifier");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 2));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 2));
      assert_eq!(first.value(), "?");
      assert_eq!(first.raw_value(), "?");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "EqualsMatcher");
      assert_eq!(*second.get_value_range(), SourceRange::new(2, 9));
      assert_eq!(*second.get_raw_range(), SourceRange::new(2, 9));
      assert_eq!(second.value(), "='test'");
      assert_eq!(second.raw_value(), "='test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new("< %'[',']','' >");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Pattern");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 15));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 15));
      assert_eq!(token.value(), "< %'[',']','' >");
      assert_eq!(token.raw_value(), "< %'[',']','' >");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "SequenceMatcher");
      assert_eq!(*first.get_value_range(), SourceRange::new(2, 13));
      assert_eq!(*first.get_raw_range(), SourceRange::new(2, 13));
      assert_eq!(first.value(), "%'[',']',''");
      assert_eq!(first.raw_value(), "%'[',']',''");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_on_not_optional() {
    let parser = Parser::new("<?!='test'>");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptPattern!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Pattern");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 15));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 15));
      assert_eq!(token.value(), "< %'[',']','' >");
      assert_eq!(token.raw_value(), "< %'[',']','' >");
      assert_eq!(token.get_children().len(), 1);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptPattern!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
