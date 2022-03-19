use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;

pub struct FlattenPattern<'a> {
  matcher: Box<dyn Matcher>,
  name: &'a str,
}

impl<'a> FlattenPattern<'a> {
  pub fn new(matcher: Box<dyn Matcher>) -> Self {
    FlattenPattern {
      matcher,
      name: "Flatten",
    }
  }

  pub fn new_with_name(matcher: Box<dyn Matcher>, name: &'a str) -> Self {
    FlattenPattern { matcher, name }
  }
}

impl<'a> Matcher for FlattenPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let result = self
      .matcher
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

  fn get_name(&self) -> &str {
    self.name
  }
}

#[macro_export]
macro_rules! Flatten {
  ($name:expr; $arg:expr) => {
    $crate::matchers::flatten::FlattenPattern::new_with_name(std::boxed::Box::new($arg), $name)
  };

  ($arg:expr) => {
    $crate::matchers::flatten::FlattenPattern::new(std::boxed::Box::new($arg))
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{Matcher, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Flatten, Loop, Matches, Switch,
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

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
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
