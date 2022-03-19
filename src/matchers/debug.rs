use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct DebugPattern {
  matcher: Option<Box<dyn Matcher>>,
  debug_mode: usize,
}

impl DebugPattern {
  pub fn new(matcher: Option<Box<dyn Matcher>>) -> Self {
    Self {
      matcher,
      debug_mode: 1,
    }
  }

  pub fn new_with_debug_mode(matcher: Option<Box<dyn Matcher>>, debug_mode: usize) -> Self {
    Self {
      matcher,
      debug_mode,
    }
  }
}

impl Matcher for DebugPattern {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let sub_context = context.clone_with_name(self.get_name());

    sub_context.borrow_mut().set_debug_mode(self.debug_mode);

    match &self.matcher {
      Some(matcher) => {
        let result = matcher.exec(sub_context.clone());
        let sub_context = sub_context.borrow();

        println!(
          "'{}' matcher at: -->|{}|--> @[{}-{}]: {:?}",
          matcher.get_name(),
          sub_context.debug_range(10),
          sub_context.offset.start,
          sub_context.offset.start + 10,
          result,
        );

        result
      }
      None => {
        println!(
          "{{Context}}: -->|{}|--> @[{}-{}], start: {}, end: {}",
          context.debug_range(10),
          context.offset.start,
          context.offset.start + 10,
          context.offset.start,
          context.offset.end,
        );

        Ok(MatcherSuccess::Skip(0))
      }
    }
  }

  fn get_name(&self) -> &str {
    "Debug"
  }
}

#[macro_export]
macro_rules! Debug {
  ($mode:expr; $arg:expr) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(
      Some(std::boxed::Box::new($arg)),
      $mode,
    )
  };

  ($mode:expr;) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(None, $mode)
  };

  ($arg:expr) => {
    $crate::matchers::debug::DebugPattern::new(Some(std::boxed::Box::new($arg)))
  };

  () => {
    $crate::matchers::debug::DebugPattern::new(None)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Debug!(Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }
}
