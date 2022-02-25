use std::cell::RefCell;
use std::rc::Rc;

use super::parser::Parser;
use super::source_range::SourceRange;

pub type TokenRef<'a> = Rc<RefCell<Token<'a>>>;

#[derive(Debug)]
pub struct Token<'a> {
  pub value_range: SourceRange,
  pub raw_range: SourceRange,
  pub name: &'a str,
  pub parent: Option<TokenRef<'a>>,
  pub children: Vec<TokenRef<'a>>,
}

impl<'a> PartialEq for Token<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.value_range == other.value_range && self.name == other.name
  }
}

impl Token<'_> {
  pub fn to_string<'p>(&self, parser: &'p Parser) -> &'p str {
    self.value_range.to_string(parser)
  }

  pub fn new<'a>(name: &'a str, value_range: SourceRange) -> TokenRef<'a> {
    Rc::new(RefCell::new(Token {
      value_range,
      raw_range: value_range.clone(),
      name,
      parent: None,
      children: Vec::new(),
    }))
  }

  pub fn new_with_raw_range<'a>(
    name: &'a str,
    value_range: SourceRange,
    raw_range: SourceRange,
  ) -> TokenRef<'a> {
    Rc::new(RefCell::new(Token {
      value_range,
      raw_range,
      name,
      parent: None,
      children: Vec::new(),
    }))
  }

  pub fn value<'a>(&self, parser: &'a Parser) -> &'a str {
    &parser.source[self.value_range.start..self.value_range.end]
  }

  pub fn raw_value<'a>(&self, parser: &'a Parser) -> &'a str {
    &parser.source[self.raw_range.start..self.raw_range.end]
  }
}
