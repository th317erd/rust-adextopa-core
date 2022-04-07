use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(std::fmt::Debug)]
pub struct DebugPattern {
  matcher: Option<MatcherRef>,
  debug_mode: usize,
}

impl DebugPattern {
  pub fn new(matcher: Option<MatcherRef>) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      matcher: match matcher {
        Some(matcher) => Some(matcher),
        None => None,
      },
      debug_mode: 1,
    })))
  }

  pub fn new_with_debug_mode(matcher: Option<MatcherRef>, debug_mode: usize) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      matcher,
      debug_mode,
    })))
  }
}

impl Matcher for DebugPattern {
  fn exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let sub_context = context.clone_with_name(self.get_name());

    sub_context.borrow_mut().set_debug_mode(self.debug_mode);

    match self.matcher {
      Some(ref matcher) => {
        let debug_range = sub_context.borrow().debug_range(10);
        let start_offset = sub_context.borrow().offset.start;
        let end_offset = sub_context.borrow().offset.end;
        let debug_mode_level = sub_context.borrow().debug_mode_level();

        let matcher = RefCell::borrow(matcher);
        let result = matcher.exec(sub_context, scope.clone());

        if debug_mode_level > 2 {
          print!("{{Debug}} ");
        }

        println!(
          "`{}` matcher at: -->|{}|--> @[{}-{}]: {:?}",
          matcher.get_name(),
          debug_range
            .as_str()
            .replace("\n", r"\n")
            .replace("\r", r"\r")
            .replace("\t", r"\t"),
          start_offset,
          std::cmp::min(start_offset + 10, end_offset),
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

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Debug` matcher");
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef) {
    if index > 0 {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.matcher = Some(matcher);
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    match &self.matcher {
      Some(matcher) => Some(vec![matcher.clone()]),
      None => None,
    }
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Debug` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Debug {
  ($mode:literal; $arg:expr) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(Some($arg), $mode)
  };

  ($mode:literal;) => {
    $crate::matchers::debug::DebugPattern::new_with_debug_mode(None, $mode)
  };

  ($arg:expr) => {
    $crate::matchers::debug::DebugPattern::new(Some($arg))
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

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }
}
