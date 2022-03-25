use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct RefPattern<'a> {
  target_name: &'a str,
}

impl<'a> RefPattern<'a> {
  pub fn new(target_name: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(RefPattern { target_name })))
  }
}

impl<'a> Matcher<'a> for RefPattern<'a> {
  fn exec(&self, _: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    panic!("`exec` called on a `Ref` pattern, which should never be executed. `Ref` matchers are only meant for reference substitution. Did you call `matcher.exec` instead of `PatternContext::tokenize`?")
  }

  fn get_name(&self) -> &str {
    "Ref"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Ref` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Ref` matcher");
  }

  fn swap_with_reference_name(&self) -> Option<&'a str> {
    Some(self.target_name)
  }
}

#[macro_export]
macro_rules! Ref {
  ($arg:expr) => {
    $crate::matchers::r#ref::RefPattern::new($arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Matches, Program,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Hello World");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Matches!("Word"; r"\w+"), Matches!(r"\s+"), Ref!("Word"));

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 11));
      assert_eq!(token.value(), "Hello World");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Word");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 5));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 5));
      assert_eq!(first.value(), "Hello");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Matches");
      assert_eq!(*second.get_value_range(), SourceRange::new(5, 6));
      assert_eq!(*second.get_raw_range(), SourceRange::new(5, 6));
      assert_eq!(second.value(), " ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Word");
      assert_eq!(*third.get_value_range(), SourceRange::new(6, 11));
      assert_eq!(*third.get_raw_range(), SourceRange::new(6, 11));
      assert_eq!(third.value(), "World");
    } else {
      unreachable!("Test failed!");
    };
  }
}
