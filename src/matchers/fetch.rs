use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::{ParserContextRef, VariableType};

pub trait Fetchable<'a> {
  fn fetch_value(&self, context: ParserContextRef) -> String;
}

impl<'a> Fetchable<'a> for FetchPattern {
  fn fetch_value(&self, context: ParserContextRef) -> String {
    let name = self.get_name();
    let name_path: Vec<&str> = name.split(".").collect();

    match context.borrow().get_variable(name_path[0]) {
      Some(VariableType::Token(ref token)) => {
        if name_path.len() != 2 {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a Token reference",
            name
          );
        }

        let token = token.borrow();
        let sub_name = name_path[1];

        if sub_name == "value" {
          token.value()
        } else if sub_name == "raw_value" {
          token.raw_value()
        } else if sub_name == "start" {
          format!("{}", token.get_raw_range().start)
        } else if sub_name == "end" {
          format!("{}", token.get_raw_range().end)
        } else if sub_name == "value_start" {
          format!("{}", token.get_value_range().start)
        } else if sub_name == "value_end" {
          format!("{}", token.get_value_range().end)
        } else if sub_name == "range" {
          let range = token.get_raw_range();
          format!("{}..{}", range.start, range.end)
        } else if sub_name == "value_range" {
          let range = token.get_value_range();
          format!("{}..{}", range.start, range.end)
        } else {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a Token reference",
            name
          );
        }
      }
      Some(VariableType::String(ref value)) => {
        if name_path.len() > 1 {
          panic!(
            "Invalid variable reference `{}`: Don't know how to fetch this value on a String reference",
            name
          );
        }

        value.clone()
      }
      None => {
        panic!("Invalid variable reference `{}`: Not found", name);
      }
    }
  }
}

impl<'a> Fetchable<'a> for &'a str {
  fn fetch_value(&self, _: ParserContextRef) -> String {
    self.to_string()
  }
}

impl<'a> Fetchable<'a> for String {
  fn fetch_value(&self, _: ParserContextRef) -> String {
    self.clone()
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
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
      assert_eq!(token.raw_value(), "Testing");
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
      Equals!(Fetch!("test.value"))
    );

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 15));
      assert_eq!(*token.get_raw_range(), SourceRange::new(0, 15));
      assert_eq!(token.value(), "Testing Testing");
      assert_eq!(token.raw_value(), "Testing Testing");

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Matches");
      assert_eq!(*first.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_raw_range(), SourceRange::new(0, 7));
      assert_eq!(first.value(), "Testing");
      assert_eq!(first.raw_value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Equals");
      assert_eq!(*second.get_value_range(), SourceRange::new(8, 15));
      assert_eq!(*second.get_raw_range(), SourceRange::new(8, 15));
      assert_eq!(second.value(), "Testing");
      assert_eq!(second.raw_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }
}
