#[macro_export]
macro_rules! ScriptRegexMatcher {
  () => {
    $crate::Map!(
      $crate::Program!("RegexMatcher";
        $crate::Equals!("RegexStart"; "/"),
        $crate::ProxyChildren!(
          $crate::Loop!("RegexCaptureLoop";
            // Match all characters up to: \, /, or [
            $crate::Matches!("Part"; r"[^/\\\[]*"),
            // Test which sequence comes next
            $crate::Switch!(
              // Is this an escape sequence?
              $crate::Matches!("Part"; r"\\."),
              // Is this the final closing / of the regex? ... if so, break
              $crate::ProxyChildren!(
                $crate::Program!(
                  $crate::Equals!("RegexEnd"; "/"),
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

        Ok(())
      }
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new(r"/test\/[chars/\]]stuff/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 24));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), r"test\/[chars/\]]stuff");
      assert_eq!(token.get_matched_value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.get_children().len(), 7);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "RegexStart");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 1));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 1));
      assert_eq!(first.get_value(), r"/");
      assert_eq!(first.get_matched_value(), r"/");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Part");
      assert_eq!(*second.get_captured_range(), SourceRange::new(1, 5));
      assert_eq!(*second.get_matched_range(), SourceRange::new(1, 5));
      assert_eq!(second.get_value(), r"test");
      assert_eq!(second.get_matched_value(), r"test");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Part");
      assert_eq!(*third.get_captured_range(), SourceRange::new(5, 7));
      assert_eq!(*third.get_matched_range(), SourceRange::new(5, 7));
      assert_eq!(third.get_value(), r"\/");
      assert_eq!(third.get_matched_value(), r"\/");

      let fourth = token.get_children()[3].borrow();
      assert_eq!(fourth.get_name(), "Part");
      assert_eq!(*fourth.get_captured_range(), SourceRange::new(8, 16));
      assert_eq!(*fourth.get_matched_range(), SourceRange::new(7, 17));
      assert_eq!(fourth.get_value(), r"chars/]");
      assert_eq!(fourth.get_matched_value(), r"[chars/\]]");

      let fifth = token.get_children()[4].borrow();
      assert_eq!(fifth.get_name(), "Part");
      assert_eq!(*fifth.get_captured_range(), SourceRange::new(17, 22));
      assert_eq!(*fifth.get_matched_range(), SourceRange::new(17, 22));
      assert_eq!(fifth.get_value(), r"stuff");
      assert_eq!(fifth.get_matched_value(), r"stuff");

      let sixth = token.get_children()[5].borrow();
      assert_eq!(sixth.get_name(), "RegexEnd");
      assert_eq!(*sixth.get_captured_range(), SourceRange::new(22, 23));
      assert_eq!(*sixth.get_matched_range(), SourceRange::new(22, 23));
      assert_eq!(sixth.get_value(), r"/");
      assert_eq!(sixth.get_matched_value(), r"/");

      let seventh = token.get_children()[6].borrow();
      assert_eq!(seventh.get_name(), "Flags");
      assert_eq!(*seventh.get_captured_range(), SourceRange::new(23, 24));
      assert_eq!(*seventh.get_matched_range(), SourceRange::new(23, 24));
      assert_eq!(seventh.get_value(), r"i");
      assert_eq!(seventh.get_matched_value(), r"i");
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

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 24));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 24));
      assert_eq!(token.get_value(), r"test\/[chars/\]]stuff");
      assert_eq!(token.get_matched_value(), r"/test\/[chars/\]]stuff/i");
      assert_eq!(token.get_children().len(), 7);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "RegexStart");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 1));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 1));
      assert_eq!(first.get_value(), r"/");
      assert_eq!(first.get_matched_value(), r"/");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Part");
      assert_eq!(*second.get_captured_range(), SourceRange::new(1, 5));
      assert_eq!(*second.get_matched_range(), SourceRange::new(1, 5));
      assert_eq!(second.get_value(), r"test");
      assert_eq!(second.get_matched_value(), r"test");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Part");
      assert_eq!(*third.get_captured_range(), SourceRange::new(5, 7));
      assert_eq!(*third.get_matched_range(), SourceRange::new(5, 7));
      assert_eq!(third.get_value(), r"\/");
      assert_eq!(third.get_matched_value(), r"\/");

      let fourth = token.get_children()[3].borrow();
      assert_eq!(fourth.get_name(), "Part");
      assert_eq!(*fourth.get_captured_range(), SourceRange::new(8, 16));
      assert_eq!(*fourth.get_matched_range(), SourceRange::new(7, 17));
      assert_eq!(fourth.get_value(), r"chars/]");
      assert_eq!(fourth.get_matched_value(), r"[chars/\]]");

      let fifth = token.get_children()[4].borrow();
      assert_eq!(fifth.get_name(), "Part");
      assert_eq!(*fifth.get_captured_range(), SourceRange::new(17, 22));
      assert_eq!(*fifth.get_matched_range(), SourceRange::new(17, 22));
      assert_eq!(fifth.get_value(), r"stuff");
      assert_eq!(fifth.get_matched_value(), r"stuff");

      let sixth = token.get_children()[5].borrow();
      assert_eq!(sixth.get_name(), "RegexEnd");
      assert_eq!(*sixth.get_captured_range(), SourceRange::new(22, 23));
      assert_eq!(*sixth.get_matched_range(), SourceRange::new(22, 23));
      assert_eq!(sixth.get_value(), r"/");
      assert_eq!(sixth.get_matched_value(), r"/");

      let seventh = token.get_children()[6].borrow();
      assert_eq!(seventh.get_name(), "Flags");
      assert_eq!(*seventh.get_captured_range(), SourceRange::new(23, 24));
      assert_eq!(*seventh.get_matched_range(), SourceRange::new(23, 24));
      assert_eq!(seventh.get_value(), r"i");
      assert_eq!(seventh.get_matched_value(), r"i");
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

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 6));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 6));
      assert_eq!(token.get_value(), r"\w+");
      assert_eq!(token.get_matched_value(), r"/\w+/i");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works4() {
    let parser = Parser::new(r"/\s+/");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRegexMatcher!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(token.get_value(), r"\s+");
      assert_eq!(token.get_matched_value(), r"/\s+/");
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
