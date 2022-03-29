use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;

use super::fetch::{Fetchable, FetchableType};

pub struct RefPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  name: String,
  scope: Option<String>,
  target: T,
  custom_name: bool,
  _phantom: PhantomData<&'a T>,
}

impl<'a, T> std::fmt::Debug for RefPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
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

impl<'a, T> RefPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  pub fn new(target: T) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(RefPattern {
      name: "Ref".to_string(),
      scope: None,
      target,
      custom_name: false,
      _phantom: PhantomData,
    })))
  }

  pub fn new_with_name(target: T, name: String) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(RefPattern {
      name,
      scope: None,
      target,
      custom_name: true,
      _phantom: PhantomData,
    })))
  }

  pub fn get_scope(&self) -> Option<&str> {
    match &self.scope {
      Some(name) => Some(name.as_str()),
      None => None,
    }
  }
}

impl<'a, T> Matcher<'a> for RefPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let target = self.target.fetch_value(sub_context.clone());

    match target {
      FetchableType::String(ref target_name) => {
        let possible_matcher = sub_context
          .borrow()
          .get_registered_matcher(self.get_scope(), target_name);

        if let Some(matcher) = possible_matcher {
          matcher.borrow().exec(sub_context)
        } else {
          return Err(MatcherFailure::Error(format!(
            "`Ref` matcher unable to locate target reference `{}`",
            target_name
          )));
        }
      }
      FetchableType::Matcher(matcher) => matcher.borrow().exec(sub_context),
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

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Ref` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }

  fn set_scope(&mut self, scope: Option<&str>) {
    match scope {
      Some(name) => self.scope = Some(name.to_string()),
      None => self.scope = None,
    }
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
