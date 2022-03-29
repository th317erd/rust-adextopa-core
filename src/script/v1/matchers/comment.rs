#[macro_export]
macro_rules! ScriptComment {
  () => {
    $crate::Matches!("Comment"; r"#[^\r\n]*")
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
    let parser = Parser::new("# Testing\n");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptComment!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Comment");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 9));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 9));
      assert_eq!(token.get_value(), "# Testing");
      assert_eq!(token.get_matched_value(), "# Testing");
      assert_eq!(token.get_children().len(), 0);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"1234 # Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(Switch!(
      ScriptComment!(),
      Matches!("Whitespace"; r"\s+"),
      Matches!("Number"; r"\d+"),
    ));

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 14));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 14));
      assert_eq!(token.get_value(), r"1234 # Testing");
      assert_eq!(token.get_matched_value(), r"1234 # Testing");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Number");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
      assert_eq!(first.get_value(), r"1234");
      assert_eq!(first.get_matched_value(), r"1234");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Whitespace");
      assert_eq!(*second.get_captured_range(), SourceRange::new(4, 5));
      assert_eq!(*second.get_matched_range(), SourceRange::new(4, 5));
      assert_eq!(second.get_value(), r" ");
      assert_eq!(second.get_matched_value(), r" ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Comment");
      assert_eq!(*third.get_captured_range(), SourceRange::new(5, 14));
      assert_eq!(*third.get_matched_range(), SourceRange::new(5, 14));
      assert_eq!(third.get_value(), r"# Testing");
      assert_eq!(third.get_matched_value(), r"# Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptComment!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
