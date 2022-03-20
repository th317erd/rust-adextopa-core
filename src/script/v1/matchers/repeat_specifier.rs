#[macro_export]
macro_rules! ScriptRepeatSpecifier {
  () => {
    $crate::Switch!("RepeatSpecifier";
      $crate::ScriptRepeatZeroOrMore!(),
      $crate::ScriptRepeatOneOrMore!(),
      $crate::ScriptRepeatRange!(),
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
    let parser = Parser::new(r"+*{}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatOneOrMore");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 1));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 1));
      assert_eq!(token.value(), r"+");
      assert_eq!(token.raw_value(), r"+");
      assert_eq!(token.get_children().len(), 0);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"*+{}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatZeroOrMore");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 1));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 1));
      assert_eq!(token.value(), r"*");
      assert_eq!(token.raw_value(), r"*");
      assert_eq!(token.get_children().len(), 0);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new(r"{10,}*+");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 5));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 5));
      assert_eq!(token.value(), r"{10,}");
      assert_eq!(token.raw_value(), r"{10,}");
      assert_eq!(token.get_children().len(), 2);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new(" +");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
