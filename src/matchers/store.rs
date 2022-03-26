use std::cell::RefCell;
use std::rc::Rc;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::{ParserContextRef, VariableType};

#[derive(Debug)]
pub enum StorePatternType<'a> {
  Matcher(MatcherRef<'a>),
  String(String),
}

#[derive(Debug)]
pub struct StorePattern<'a> {
  pattern: StorePatternType<'a>,
  name: String,
  custom_name: bool,
}

impl<'a> StorePattern<'a> {
  pub fn new_as_string_type(name: &'a str, pattern: &'a str) -> MatcherRef<'a> {
    if let Some(_) = name.find(".") {
      panic!("`Store`: Variable names can not contain `.` characters");
    }

    Rc::new(RefCell::new(Box::new(StorePattern {
      pattern: StorePatternType::String(pattern.to_string()),
      name: name.to_string(),
      custom_name: true,
    })))
  }

  pub fn new_as_matcher_type(name: &'a str, pattern: MatcherRef<'a>) -> MatcherRef<'a> {
    if let Some(_) = name.find(".") {
      panic!("`Store`: Variable names can not contain `.` characters");
    }

    Rc::new(RefCell::new(Box::new(StorePattern {
      pattern: StorePatternType::Matcher(pattern),
      name: name.to_string(),
      custom_name: true,
    })))
  }
}

impl<'a> Matcher<'a> for StorePattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    match &self.pattern {
      StorePatternType::Matcher(matcher) => {
        let sub_context = context.borrow().clone_with_name(self.get_name());
        let result = matcher.borrow().exec(sub_context);

        match result {
          Ok(MatcherSuccess::Token(ref token)) => {
            context
              .borrow_mut()
              .set_variable(self.name.to_string(), VariableType::Token(token.clone()));
          }
          _ => {}
        }

        result
      }
      StorePatternType::String(value) => {
        context
          .borrow_mut()
          .set_variable(self.name.to_string(), VariableType::String(value.clone()));

        Ok(MatcherSuccess::Skip(0))
      }
    }
  }

  fn is_consuming(&self) -> bool {
    match &self.pattern {
      StorePatternType::Matcher(_) => true,
      StorePatternType::String(_) => false,
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
    match &self.pattern {
      StorePatternType::Matcher(matcher) => Some(vec![matcher.clone()]),
      StorePatternType::String(_) => None,
    }
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Store` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Store {
  ($name:literal; $arg:literal) => {
    $crate::matchers::store::StorePattern::new_as_string_type($name, $arg)
  };

  ($name:literal; $arg:expr) => {
    $crate::matchers::store::StorePattern::new_as_matcher_type($name, $arg)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess,
    parser::Parser,
    parser_context::{ParserContext, VariableType},
    source_range::SourceRange,
    Equals, Switch,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Store!("test"; Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");

      let variable = parser_context.borrow().get_variable("test");
      if let Some(VariableType::Token(variable_token)) = variable {
        let variable_token = variable_token.borrow();
        assert_eq!(variable_token.get_name(), "Equals");
        assert_eq!(*variable_token.get_value_range(), SourceRange::new(0, 7));
        assert_eq!(*variable_token.get_raw_range(), SourceRange::new(0, 7));
        assert_eq!(variable_token.value(), "Testing");
        assert_eq!(variable_token.raw_value(), "Testing");
      } else {
        unreachable!("Test failed!");
      }
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Switch!(Store!("test"; "This is a test!"), Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");

      let variable = parser_context.borrow().get_variable("test");
      if let Some(VariableType::String(value)) = variable {
        assert_eq!(value, "This is a test!");
      } else {
        unreachable!("Test failed!");
      }
    } else {
      unreachable!("Test failed!");
    };
  }
}
