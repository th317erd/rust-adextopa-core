#[macro_export]
macro_rules! ScriptRegexMatcher {
  () => {
    $crate::Map!(
      $crate::Program!("RegexMatcher";
        $crate::Discard!($crate::Equals!("/")),
        $crate::Flatten!(
          $crate::Loop!("RegexCaptureLoop";
            // Match all characters up to: \, /, or [
            $crate::Matches!("Part"; r"[^/\\\[]*"),
            // Test which sequence comes next
            $crate::Switch!(
              // Is this an escape sequence?
              $crate::Matches!("Part"; r"\\."),
              // Is this the final closing / of the regex? ... if so, break
              $crate::Flatten!(
                $crate::Program!(
                  $crate::Discard!($crate::Equals!("/")),
                  $crate::Matches!("Flags"; r"[imsU]*"),
                  $crate::Break!("RegexCaptureLoop"),
                )
              ),
              // Is this a character sequence?
              $crate::Sequence!("Part"; "[", "]", "\\"),
            )
          )
        ),
      ),
      |token| {
        let mut range = $crate::source_range::SourceRange::new(usize::MAX, usize::MAX);

        {
          let token = token.borrow();

          for child in token.get_children() {
            let child = child.borrow();

            if child.get_name() == "Part" {
              let matched_range = child.get_matched_range();

              if range.start == usize::MAX || matched_range.start < range.start {
                range.start = matched_range.start;
              }

              if range.end == usize::MAX || matched_range.end > range.end {
                range.end = matched_range.end;
              }
            }
          }
        }

        let parser = token.borrow().get_parser();
        token.borrow_mut().set_value(&range.to_string(&parser));

        None
      }
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
    let parser = Parser::new(r"/test\/[chars/\]]stuff/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 24));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), r"test\/[chars/\]]stuff");
      assert_eq!(token.get_matched_value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.get_children().len(), 5);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Part");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 5));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 5));
      assert_eq!(first.get_value(), r"test");
      assert_eq!(first.get_matched_value(), r"test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Part");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 7));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 7));
      assert_eq!(second.get_value(), r"\/");
      assert_eq!(second.get_matched_value(), r"\/");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Part");
      assert_eq!(*third.get_captured_range(), SourceRange::new(8, 16));
      assert_eq!(*third.get_matched_range(), SourceRange::new(7, 17));
      assert_eq!(third.get_value(), r"chars/]");
      assert_eq!(third.get_matched_value(), r"[chars/\]]");

      let forth = token.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "Part");
      assert_eq!(*forth.get_captured_range(), SourceRange::new(17, 22));
      assert_eq!(*forth.get_matched_range(), SourceRange::new(17, 22));
      assert_eq!(forth.get_value(), r"stuff");
      assert_eq!(forth.get_matched_value(), r"stuff");

      let fifth = token.get_children()[4].borrow();
      assert_eq!(fifth.get_name(), "Flags");
      assert_eq!(*fifth.get_captured_range(), SourceRange::new(23, 24));
      assert_eq!(*fifth.get_matched_range(), SourceRange::new(23, 24));
      assert_eq!(fifth.get_value(), r"i");
      assert_eq!(fifth.get_matched_value(), r"i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"/test\/[chars/\]]stuff/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 24));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), r"test\/[chars/\]]stuff");
      assert_eq!(token.get_matched_value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.get_children().len(), 5);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Part");
      assert_eq!(*first.get_captured_range(), SourceRange::new(1, 5));
      assert_eq!(*first.get_matched_range(), SourceRange::new(1, 5));
      assert_eq!(first.get_value(), r"test");
      assert_eq!(first.get_matched_value(), r"test");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Part");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 7));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 7));
      assert_eq!(second.get_value(), r"\/");
      assert_eq!(second.get_matched_value(), r"\/");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Part");
      assert_eq!(*third.get_captured_range(), SourceRange::new(8, 16));
      assert_eq!(*third.get_matched_range(), SourceRange::new(7, 17));
      assert_eq!(third.get_value(), r"chars/]");
      assert_eq!(third.get_matched_value(), r"[chars/\]]");

      let forth = token.get_children()[3].borrow();
      assert_eq!(forth.get_name(), "Part");
      assert_eq!(*forth.get_captured_range(), SourceRange::new(17, 22));
      assert_eq!(*forth.get_matched_range(), SourceRange::new(17, 22));
      assert_eq!(forth.get_value(), r"stuff");
      assert_eq!(forth.get_matched_value(), r"stuff");

      let fifth = token.get_children()[4].borrow();
      assert_eq!(fifth.get_name(), "Flags");
      assert_eq!(*fifth.get_captured_range(), SourceRange::new(23, 24));
      assert_eq!(*fifth.get_matched_range(), SourceRange::new(23, 24));
      assert_eq!(fifth.get_value(), r"i");
      assert_eq!(fifth.get_matched_value(), r"i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new(r"/\w+/i");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 6));
      assert_eq!(token.get_value(), r"\w+");
      assert_eq!(token.get_matched_value(), r"/\w+/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
