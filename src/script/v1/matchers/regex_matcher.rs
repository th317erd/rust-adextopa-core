#[macro_export]
macro_rules! ScriptRegexMatcher {
  () => {
    $crate::Program!("RegexMatcher";
      $crate::Discard!($crate::Equals!("/")),
      $crate::Flatten!($crate::Loop!(
        "RegexCaptureLoop";
        // Match all characters up to: \, /, or [
        $crate::Matches!("Part"; r"[^/\\\[]*"),
        // Test which sequence comes next
        $crate::Switch!(
          // Is this an escape sequence?
          $crate::Matches!("Part"; r"\\."),
          // Is this the final closing / of the regex? ... if so, break
          $crate::Flatten!($crate::Program!(
            $crate::Discard!($crate::Equals!("/")),
            $crate::Matches!("Flags"; r"[im]*"),
            $crate::Break!("RegexCaptureLoop"),
          )),
          // Is this a character sequence?
          $crate::Sequence!("Part"; "[", "]", "\\"),
        )
      )),
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
    let parser = Parser::new(r"/test\/[chars/\]]stuff/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 24));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 24));
      assert_eq!(token.value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.raw_value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.get_children().len(), 5);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Part");
      assert_eq!(*first.get_value_range(), SourceRange::new(1, 5));
      assert_eq!(*first.get_raw_range(), SourceRange::new(1, 5));
      assert_eq!(first.value(), r"test");
      assert_eq!(first.raw_value(), r"test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Part");
      assert_eq!(*second.get_value_range(), SourceRange::new(5, 7));
      assert_eq!(*second.get_raw_range(), SourceRange::new(5, 7));
      assert_eq!(second.value(), r"\/");
      assert_eq!(second.raw_value(), r"\/");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Part");
      assert_eq!(*third.get_value_range(), SourceRange::new(8, 16));
      assert_eq!(*third.get_raw_range(), SourceRange::new(7, 17));
      assert_eq!(third.value(), r"chars/]");
      assert_eq!(third.raw_value(), r"[chars/\]]");

      let forth = token.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "Part");
      assert_eq!(*forth.get_value_range(), SourceRange::new(17, 22));
      assert_eq!(*forth.get_raw_range(), SourceRange::new(17, 22));
      assert_eq!(forth.value(), r"stuff");
      assert_eq!(forth.raw_value(), r"stuff");

      let fifth = token.get_children()[4].borrow();
      assert_eq!(fifth.get_name(), "Flags");
      assert_eq!(*fifth.get_value_range(), SourceRange::new(23, 24));
      assert_eq!(*fifth.get_raw_range(), SourceRange::new(23, 24));
      assert_eq!(fifth.value(), r"i");
      assert_eq!(fifth.raw_value(), r"i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}