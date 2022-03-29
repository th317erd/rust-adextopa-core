use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::{ParserContextRef, VariableType};

pub enum FetchableType<'a> {
  String(String),
  Matcher(MatcherRef<'a>),
}

pub trait Fetchable<'a> {
  fn fetch_value(&self, context: ParserContextRef) -> FetchableType<'a>;
}

impl<'a> Fetchable<'a> for FetchPattern {
  fn fetch_value(&self, context: ParserContextRef) -> FetchableType<'a> {
    let name = self.get_name();
    let name_path: Vec<&str> = name.split(".").collect();

    match context
      .borrow()
      .get_variable(self.get_scope(), name_path[0])
    {
      Some(VariableType::Token(ref token)) => {
        if name_path.len() != 2 {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a Token reference",
            name
          );
        }

        let token = token.borrow();
        let sub_name = name_path[1];

        let value = if sub_name == "captured_value" {
          token.get_value().clone()
        } else if sub_name == "matched_value" {
          token.get_matched_value().clone()
        } else if sub_name == "start" {
          format!("{}", token.get_matched_range().start)
        } else if sub_name == "end" {
          format!("{}", token.get_matched_range().end)
        } else if sub_name == "captured_start" {
          format!("{}", token.get_captured_range().start)
        } else if sub_name == "captured_end" {
          format!("{}", token.get_captured_range().end)
        } else if sub_name == "range" {
          let range = token.get_matched_range();
          format!("{}..{}", range.start, range.end)
        } else if sub_name == "captured_range" {
          let range = token.get_captured_range();
          format!("{}..{}", range.start, range.end)
        } else {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a Token reference",
            name
          );
        };

        FetchableType::String(value)
      }
      Some(VariableType::String(ref value)) => {
        if name_path.len() > 1 {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a String reference",
            name
          );
        }

        FetchableType::String(value.clone())
      }
      None => {
        panic!("Invalid variable reference `{}`: Not found", name);
      }
    }
  }
}

impl<'a> Fetchable<'a> for &'a str {
  fn fetch_value(&self, _: ParserContextRef) -> FetchableType<'a> {
    FetchableType::String(self.to_string())
  }
}

impl<'a> Fetchable<'a> for String {
  fn fetch_value(&self, _: ParserContextRef) -> FetchableType<'a> {
    FetchableType::String(self.clone())
  }
}

impl<'a> Fetchable<'a> for &String {
  fn fetch_value(&self, _: ParserContextRef) -> FetchableType<'a> {
    FetchableType::String((*self).clone())
  }
}

impl<'a> Fetchable<'a> for MatcherRef<'a> {
  fn fetch_value(&self, _: ParserContextRef) -> FetchableType<'a> {
    FetchableType::Matcher(self.clone())
  }
}

#[derive(Debug)]
pub struct FetchPattern {
  name: String,
  scope: Option<String>,
}

impl FetchPattern {
  pub fn new(name: &str) -> Self {
    FetchPattern {
      name: name.to_string(),
      scope: None,
    }
  }

  fn set_scope(&mut self, scope: Option<&str>) {
    match scope {
      Some(scope) => self.scope = Some(scope.to_string()),
      None => self.scope = None,
    }
  }

  pub fn get_scope(&self) -> Option<&str> {
    match &self.scope {
      Some(name) => Some(name.as_str()),
      None => None,
    }
  }
}

impl<'a> Matcher<'a> for FetchPattern {
  fn exec(&self, _: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Skip(0))
  }

  fn is_consuming(&self) -> bool {
    false
  }

  fn has_custom_name(&self) -> bool {
    false
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Fetch` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }

  fn set_scope(&mut self, scope: Option<&str>) {
    self.set_scope(scope)
  }

  fn get_scope(&self) -> Option<&str> {
    self.get_scope()
  }
}

#[macro_export]
macro_rules! Fetch {
  ($name:literal) => {
    $crate::matchers::fetch::FetchPattern::new($name)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Discard, Equals, Matches, Program, Store, Switch,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Switch!(Store!("test"; "Testing"), Equals!(Fetch!("test")));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
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

  #[test]
  fn it_works2() {
    let parser = Parser::new("Testing Testing");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Store!("test"; Matches!(r"\w+")),
      Discard!(Matches!(r"\s+")),
      Equals!(Fetch!("test.captured_value"))
    );

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 15));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 15));
      assert_eq!(token.get_value(), "Testing Testing");
      assert_eq!(token.get_matched_value(), "Testing Testing");

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Matches");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "Testing");
      assert_eq!(first.get_matched_value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Equals");
      assert_eq!(*second.get_captured_range(), SourceRange::new(8, 15));
      assert_eq!(*second.get_matched_range(), SourceRange::new(8, 15));
      assert_eq!(second.get_value(), "Testing");
      assert_eq!(second.get_matched_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }
}
