use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct DebugPattern<'a> {
  matcher: Option<MatcherRef<'a>>,
  debug_mode: usize,
}

impl<'a> DebugPattern<'a> {
  pub fn new(matcher: Option<MatcherRef<'a>>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      matcher: match matcher {
        Some(matcher) => Some(matcher),
        None => None,
      },
      debug_mode: 1,
    })))
  }

  pub fn new_with_debug_mode(matcher: Option<MatcherRef<'a>>, debug_mode: usize) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self {
      matcher,
      debug_mode,
    })))
  }
}

impl<'a> Matcher<'a> for DebugPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let sub_context = context.clone_with_name(self.get_name());

    sub_context.borrow_mut().set_debug_mode(self.debug_mode);

    match self.matcher {
      Some(ref matcher) => {
        let debug_range = sub_context.borrow().debug_range(10);
        let start_offset = sub_context.borrow().offset.start;

        let matcher = RefCell::borrow(matcher);
        let result = matcher.exec(sub_context);

        println!(
          "'{}' matcher at: -->|{}|--> @[{}-{}]: {:?}",
          matcher.get_name(),
          debug_range,
          start_offset,
          start_offset + 10,
          result,
        );

        let result = result.clone();
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

  fn set_name(&mut self, _: &'a str) {
    panic!("Can not set 'name' on a Debug pattern");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    match &self.matcher {
      Some(matcher) => Some(vec![matcher.clone()]),
      None => None,
    }
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a Debug pattern");
  }
}

#[macro_export]
macro_rules! Debug {
  ($mode:expr; $arg:expr) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(Some($arg.clone()), $mode)
  };

  ($mode:expr;) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(None, $mode)
  };

  ($arg:expr) => {
    $crate::matchers::debug::DebugPattern::new(Some($arg.clone()))
  };

  () => {
    $crate::matchers::debug::DebugPattern::new(None)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Equals,
  };

  #[test]
  fn it_matches_against_a_string() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Debug!(Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.borrow().exec(parser_context.clone()) {
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
