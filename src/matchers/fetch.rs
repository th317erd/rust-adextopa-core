use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope::VariableType;
use crate::scope_context::ScopeContextRef;

pub enum FetchableType {
  String(String),
  Matcher(MatcherRef),
}

pub trait Fetchable {
  fn fetch_value(&self, context: ParserContextRef, scope: ScopeContextRef) -> FetchableType;
}

impl Fetchable for FetchPattern {
  fn fetch_value(&self, _: ParserContextRef, scope: ScopeContextRef) -> FetchableType {
    let name = self.get_name();
    let name_path: Vec<&str> = name.split(".").collect();

    match scope.borrow().get(name_path[0]) {
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
      Some(VariableType::Matcher(matcher)) => FetchableType::Matcher(matcher.clone()),
      None => {
        panic!("Invalid variable reference `{}`: Not found", name);
      }
    }
  }
}

impl Fetchable for &str {
  fn fetch_value(&self, _: ParserContextRef, _: ScopeContextRef) -> FetchableType {
    FetchableType::String(self.to_string())
  }
}

impl Fetchable for String {
  fn fetch_value(&self, _: ParserContextRef, _: ScopeContextRef) -> FetchableType {
    FetchableType::String(self.clone())
  }
}

impl Fetchable for &String {
  fn fetch_value(&self, _: ParserContextRef, _: ScopeContextRef) -> FetchableType {
    FetchableType::String((*self).clone())
  }
}

impl Fetchable for MatcherRef {
  fn fetch_value(&self, _: ParserContextRef, _: ScopeContextRef) -> FetchableType {
    FetchableType::Matcher(self.clone())
  }
}

#[derive(Debug)]
pub struct FetchPattern {
  name: String,
}

impl FetchPattern {
  pub fn new(name: &str) -> Self {
    FetchPattern {
      name: name.to_string(),
    }
  }

  fn _exec(
    &self,
    _: ParserContextRef,
    _: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    Ok(MatcherSuccess::Skip(0))
  }
}

impl Matcher for FetchPattern {
  fn exec(
    &self,
    this_matcher: MatcherRef,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    self.before_exec(this_matcher.clone(), context.clone(), scope.clone());
    let result = self._exec(context.clone(), scope.clone());
    self.after_exec(this_matcher.clone(), context.clone(), scope.clone());

    result
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

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    None
  }

  fn add_pattern(&mut self, _: MatcherRef) {
    panic!("Can not add a pattern to a `Fetch` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
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
    parser::Parser, parser_context::ParserContext, source_range::SourceRange, Discard, Equals,
    Matches, Program, Store, Switch,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Switch!(Store!("test"; "Testing"), Equals!(Fetch!("test")));

    if let Ok(token) = ParserContext::tokenize(parser_context.clone(), matcher) {
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

    if let Ok(token) = ParserContext::tokenize(parser_context.clone(), matcher) {
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
