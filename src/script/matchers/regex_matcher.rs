#[macro_export]
macro_rules! ScriptRegexMatcher {
  () => {
    $crate::Program!(
      "RegexMatcher";
      $crate::Discard!($crate::Equals!("/")),
      $crate::Loop!(
        "RegexCaptureLoop";
        // Match all characters up to: \, /, or [
        $crate::Matches!(r"[^/\\\[]*"),
        // Test which sequence comes next
        $crate::Switch!(
          // Is this an escape sequence?
          $crate::Matches!(r"\\."),
          // Is this the final closing / of the regex? ... if so, break
          $crate::Program!(
            $crate::Equals!("/"),
            $crate::Break!("RegexCaptureLoop"),
          ),
          // Is this a character sequence?
          $crate::Sequence!("[", "]", "\\"),
        )
      ),
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
    let parser = Parser::new(r"/test\/[chars/\]]stuff/");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptRegexMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "RegexMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 23));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 23));
      assert_eq!(token.value(), r"/test\/[chars/\]]stuff/");
      assert_eq!(token.raw_value(), r"/test\/[chars/\]]stuff/");
      // assert_eq!(token.get_children().len(), 1);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing");
    let parser_context = ParserContext::new(&parser);
    let matcher = ScriptRegexMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
