#[macro_export]
macro_rules! ScriptBaseSwitchMatcher {
  () => {
    $crate::Program!("SwitchMatcher";
      $crate::Discard!($crate::Equals!("[")),
      $crate::Flatten!(
        $crate::Loop!(
          $crate::ScriptWSN0!(?),
          $crate::ScriptMatcher!(),
          $crate::ScriptWSN0!(?),
          $crate::Switch!(
            $crate::Discard!($crate::Equals!(",")),
            $crate::Program!(
              $crate::Discard!($crate::Equals!("]")),
              $crate::Break!(),
            )
          )
        )
      )
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
    let parser = Parser::new("[]");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptBaseSwitchMatcher!();

    let result = matcher.exec(parser_context.clone());

    if let Ok(MatcherSuccess::Token(token)) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "CustomMatcher");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 4));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 4));
      assert_eq!(token.value(), "test");
      assert_eq!(token.raw_value(), "test");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("<test>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptBaseSwitchMatcher!();

    if let Err(MatcherFailure::Fail) = matcher.exec(parser_context.clone()) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
