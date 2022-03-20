#[macro_export]
macro_rules! ScriptRepeatRange {
  () => {
    $crate::Program!("RepeatRange";
      $crate::Discard!($crate::Equals!("{")),
      $crate::ScriptWS0!(?),
      $crate::Matches!("Minimum"; r"\d+"),
      $crate::ScriptWS0!(?),
      $crate::Optional!($crate::Equals!("Sep"; ",")),
      $crate::ScriptWS0!(?),
      $crate::Optional!($crate::Matches!("Maximum"; r"\d+")),
      $crate::ScriptWS0!(?),
      $crate::Discard!($crate::Equals!("}")),
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
    let parser = Parser::new(r"{10}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatRange!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 4));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 4));
      assert_eq!(token.value(), r"{10}");
      assert_eq!(token.raw_value(), r"{10}");
      assert_eq!(token.get_children().len(), 1);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Minimum");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 3));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 3));
      assert_eq!(first.value(), "10");
      assert_eq!(first.raw_value(), "10");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"{ 9, 11 }");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatRange!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 9));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 9));
      assert_eq!(token.value(), r"{ 9, 11 }");
      assert_eq!(token.raw_value(), r"{ 9, 11 }");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Minimum");
      assert_eq!(*first.get_value_range(), SourceRange::new(2, 3));
      assert_eq!(*first.get_raw_range(), SourceRange::new(2, 3));
      assert_eq!(first.value(), "9");
      assert_eq!(first.raw_value(), "9");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Sep");
      assert_eq!(*second.get_value_range(), SourceRange::new(3, 4));
      assert_eq!(*second.get_raw_range(), SourceRange::new(3, 4));
      assert_eq!(second.value(), ",");
      assert_eq!(second.raw_value(), ",");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Maximum");
      assert_eq!(*third.get_value_range(), SourceRange::new(5, 7));
      assert_eq!(*third.get_raw_range(), SourceRange::new(5, 7));
      assert_eq!(third.value(), "11");
      assert_eq!(third.raw_value(), "11");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new(r"{19,}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatRange!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 5));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 5));
      assert_eq!(token.value(), r"{19,}");
      assert_eq!(token.raw_value(), r"{19,}");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Minimum");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 3));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 3));
      assert_eq!(first.value(), "19");
      assert_eq!(first.raw_value(), "19");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Sep");
      assert_eq!(*second.get_value_range(), SourceRange::new(3, 4));
      assert_eq!(*second.get_raw_range(), SourceRange::new(3, 4));
      assert_eq!(second.value(), ",");
      assert_eq!(second.raw_value(), ",");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatRange!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
