use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope::VariableType;
use crate::scope_context::ScopeContextRef;

use super::fetch::{Fetchable, FetchableType};

pub struct RefPattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  name: String,
  target: T,
  custom_name: bool,
}

impl<T> std::fmt::Debug for RefPattern<T>
where
  T: Fetchable,
  T: std::fmt::Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RefPattern")
      .field("name", &self.name)
      .field("target", &self.target)
      .field("custom_name", &self.custom_name)
      .finish()
  }
}

impl<T> RefPattern<T>
where
  T: Fetchable,
  T: 'static,
  T: std::fmt::Debug,
{
  pub fn new(target: T) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(RefPattern {
      name: "Ref".to_string(),
      target,
      custom_name: false,
    })))
  }

  pub fn new_with_name(target: T, name: String) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(RefPattern {
      name,
      target,
      custom_name: true,
    })))
  }
}

impl<T> Matcher for RefPattern<T>
where
  T: Fetchable,
  T: 'static,
  T: std::fmt::Debug,
{
  fn exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let target = self.target.fetch_value(sub_context.clone(), scope.clone());

    match target {
      FetchableType::String(ref target_name) => {
        println!("Fetching reference {}", target_name);

        let possible_matcher = scope.borrow().get(target_name);

        if let Some(VariableType::Matcher(ref matcher)) = possible_matcher {
          matcher.borrow().exec(sub_context, scope.clone())
        } else {
          return Err(MatcherFailure::Error(format!(
            "`Ref` matcher unable to locate target reference `{}`",
            target_name
          )));
        }
      }
      FetchableType::Matcher(matcher) => matcher.borrow().exec(sub_context, scope.clone()),
    }
  }

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
    self.custom_name = true;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Ref` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Ref {
  ($name:expr; $arg:expr) => {
    $crate::matchers::r#ref::RefPattern::new_with_name($arg, $name)
  };

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
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_value(), "Hello World");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Word");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 5));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(first.get_value(), "Hello");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Matches");
      assert_eq!(*second.get_captured_range(), SourceRange::new(5, 6));
      assert_eq!(*second.get_matched_range(), SourceRange::new(5, 6));
      assert_eq!(second.get_value(), " ");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Word");
      assert_eq!(*third.get_captured_range(), SourceRange::new(6, 11));
      assert_eq!(*third.get_matched_range(), SourceRange::new(6, 11));
      assert_eq!(third.get_value(), "World");
    } else {
      unreachable!("Test failed!");
    };
  }
}
