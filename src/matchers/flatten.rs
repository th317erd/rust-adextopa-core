use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FlattenPattern<'a> {
  matcher: MatcherRef<'a>,
  name: &'a str,
  custom_name: bool,
}

impl<'a> FlattenPattern<'a> {
  pub fn new(matcher: MatcherRef<'a>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(FlattenPattern {
      matcher,
      name: "Flatten",
      custom_name: false,
    })))
  }

  pub fn new_with_name(matcher: MatcherRef<'a>, name: &'a str) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(FlattenPattern {
      matcher,
      name,
      custom_name: true,
    })))
  }
}

impl<'a> Matcher<'a> for FlattenPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let result = self
      .matcher
      .borrow()
      .exec(context.borrow().clone_with_name(self.get_name()));

    match result {
      Ok(MatcherSuccess::Token(token)) => Ok(MatcherSuccess::ExtractChildren(token.clone())),
      Ok(MatcherSuccess::Break((loop_name, value))) => match *value {
        MatcherSuccess::Token(token) => Ok(MatcherSuccess::Break((
          loop_name,
          Box::new(MatcherSuccess::ExtractChildren(token.clone())),
        ))),
        _ => Ok(MatcherSuccess::Break((loop_name, value))),
      },
      Ok(MatcherSuccess::Continue((loop_name, value))) => match *value {
        MatcherSuccess::Token(token) => Ok(MatcherSuccess::Continue((
          loop_name,
          Box::new(MatcherSuccess::ExtractChildren(token.clone())),
        ))),
        _ => Ok(MatcherSuccess::Continue((loop_name, value))),
      },
      _ => result,
    }
  }

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name
  }

  fn set_name(&mut self, name: &'a str) {
    self.name = name;
    self.custom_name = true;
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef<'a>) {
    if index > 0 {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.matcher = matcher;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    Some(vec![self.matcher.clone()])
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Flatten` matcher");
  }
}

#[macro_export]
macro_rules! Flatten {
  ($name:literal; $arg:expr) => {
    $crate::matchers::flatten::FlattenPattern::new_with_name($arg.clone(), $name)
  };

  ($arg:expr) => {
    $crate::matchers::flatten::FlattenPattern::new($arg.clone())
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Flatten, Loop, Matches, Switch,
  };

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(
      "Loop";
      Flatten!(Loop!(
        Switch!(
          Matches!("Whitespace"; r"\s+"),
          Matches!("Word"; r"[a-zA-Z_]\w+"),
          Matches!("Number"; r"\d+")
        )
      ))
    );

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), "Testing 1234");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Word");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(first.value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Whitespace");
      assert_eq!(*second.get_value_range(), SourceRange::new(7, 8));
      assert_eq!(*second.get_raw_range(), SourceRange::new(7, 8));
      assert_eq!(second.value(), " ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Number");
      assert_eq!(*third.get_value_range(), SourceRange::new(8, 12));
      assert_eq!(*third.get_raw_range(), SourceRange::new(8, 12));
      assert_eq!(third.value(), "1234");
    } else {
      unreachable!("Test failed!");
    };
  }
}
