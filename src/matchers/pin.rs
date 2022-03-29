extern crate adextopa_macros;
use std::cell::RefCell;
use std::rc::Rc;

use super::fetch::{Fetchable, FetchableType};
use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser::ParserRef;
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::{Token, TokenRef};

lazy_static::lazy_static! {
  static ref EMPTY_OFFSET: regex::Regex = regex::Regex::new(r"^(\+|-|\+0|-0)$").expect("Could not compile needed Regex for `PinPattern`");
}

fn is_empty_offset(offset_str: &str) -> bool {
  if offset_str.len() == 0 {
    return true;
  }

  match EMPTY_OFFSET.find(offset_str) {
    Some(_) => true,
    None => false,
  }
}

fn parse_offset(start: usize, end: usize, offset_str: &str) -> usize {
  if !is_empty_offset(offset_str) {
    let prefix = &offset_str[0..1];
    let part: &str;

    let has_prefix = if prefix == "+" || prefix == "-" {
      part = &offset_str[1..];
      true
    } else {
      part = offset_str;
      false
    };

    match part.parse::<usize>() {
      Ok(offset) => {
        if has_prefix {
          if prefix == "-" {
            start - offset
          } else {
            start + offset
          }
        } else {
          offset
        }
      }
      Err(err) => {
        panic!(
          "Error while attempting to parse `Pin` start offset [{}]: {}",
          offset_str, err
        );
      }
    }
  } else {
    end
  }
}

#[derive(Debug)]
pub struct PinToken {
  parser: ParserRef,
  pub captured_range: SourceRange,
  pub matched_range: SourceRange,
  pub name: String,
  pub value: Option<String>,
  pub captured_value: String,
  pub matched_value: String,
  pub parent: Option<TokenRef>,
  pub children: Vec<TokenRef>,
  pub attributes: std::collections::HashMap<String, String>,
}

impl PinToken {
  pub fn new(parser: &ParserRef, name: String, captured_range: SourceRange) -> TokenRef {
    Rc::new(RefCell::new(Box::new(Self {
      parser: parser.clone(),
      captured_range,
      matched_range: captured_range.clone(),
      name,
      value: None,
      captured_value: captured_range.to_string(&parser),
      matched_value: captured_range.to_string(&parser),
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }

  pub fn new_with_matched_range(
    parser: &ParserRef,
    name: String,
    captured_range: SourceRange,
    matched_range: SourceRange,
  ) -> TokenRef {
    Rc::new(RefCell::new(Box::new(Self {
      parser: parser.clone(),
      captured_range,
      matched_range,
      name,
      value: None,
      captured_value: captured_range.to_string(&parser),
      matched_value: matched_range.to_string(&parser),
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }
}

impl Token for PinToken {
  fn get_parser(&self) -> crate::parser::ParserRef {
    self.parser.clone()
  }

  fn get_captured_range(&self) -> &crate::source_range::SourceRange {
    &self.captured_range
  }

  fn get_captured_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
    &mut self.captured_range
  }

  fn set_captured_range(&mut self, range: crate::source_range::SourceRange) {
    self.captured_range = range;
  }

  fn get_matched_range(&self) -> &crate::source_range::SourceRange {
    &self.matched_range
  }

  fn get_matched_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
    &mut self.matched_range
  }

  fn set_matched_range(&mut self, range: crate::source_range::SourceRange) {
    self.matched_range = range;
  }

  fn get_name(&self) -> &String {
    &self.name
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
  }

  fn get_parent(&self) -> Option<crate::token::TokenRef> {
    match self.parent {
      Some(ref token_ref) => Some(token_ref.clone()),
      None => None,
    }
  }

  fn set_parent(&mut self, token: Option<crate::token::TokenRef>) {
    self.parent = token;
  }

  fn get_children<'b>(&'b self) -> &'b Vec<crate::token::TokenRef> {
    &self.children
  }

  fn get_children_mut<'b>(&'b mut self) -> &'b mut Vec<crate::token::TokenRef> {
    &mut self.children
  }

  fn set_children(&mut self, children: Vec<crate::token::TokenRef>) {
    self.children = children;
  }

  fn get_value(&self) -> &String {
    // Value override via attribute
    match self.get_attribute("__value") {
      Some(value) => {
        return value;
      }
      None => {}
    }

    match &self.value {
      Some(ref value) => value,
      None => self.get_captured_value(),
    }
  }

  fn set_value(&mut self, value: &str) {
    if value == "" {
      self.value = None;
    } else {
      self.value = Some(value.to_string());
    }
  }

  fn get_captured_value(&self) -> &String {
    // Value override via attribute
    match self.get_attribute("__value") {
      Some(value) => {
        return value;
      }
      None => {}
    }

    &self.captured_value
  }

  fn set_captured_value(&mut self, value: &str) {
    self.captured_value = value.to_string();
  }

  fn get_matched_value(&self) -> &String {
    // Value override via attribute
    match self.get_attribute("__matched_value") {
      Some(value) => {
        return value;
      }
      None => {}
    }

    &self.matched_value
  }

  fn set_matched_value(&mut self, value: &str) {
    self.matched_value = value.to_string();
  }

  fn get_attributes<'b>(&'b self) -> &'b std::collections::HashMap<String, String> {
    &self.attributes
  }

  fn get_attribute<'b>(&'b self, name: &str) -> Option<&'b String> {
    self.attributes.get(&name.to_string())
  }

  fn attribute_equals<'b>(&'b self, name: &str, value: &str) -> bool {
    match self.attributes.get(&name.to_string()) {
      Some(v) => (v == value),
      None => false,
    }
  }

  fn set_attribute(&mut self, name: &str, value: &str) -> Option<String> {
    self.attributes.insert(name.to_string(), value.to_string())
  }

  fn should_discard(&self) -> bool {
    true
  }
}

#[derive(Debug)]
pub struct PinPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  pattern: Option<MatcherRef<'a>>,
  offset: T,
}

impl<'a, T> PinPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  pub fn new(offset: T, pattern: Option<MatcherRef<'a>>) -> MatcherRef<'a> {
    Rc::new(RefCell::new(Box::new(Self { pattern, offset })))
  }
}

impl<'a, T> Matcher<'a> for PinPattern<'a, T>
where
  T: Fetchable<'a>,
  T: 'a,
  T: std::fmt::Debug,
{
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let sub_context = context.borrow().clone_with_name(self.get_name());
    let offset_value_fetchable = self.offset.fetch_value(sub_context.clone());
    let offset_value = match offset_value_fetchable {
      FetchableType::String(value) => value,
      FetchableType::Matcher(_) => return Err(MatcherFailure::Error(
        "`Pin` matcher received another matcher as an offset... this makes no sense... aborting..."
          .to_string(),
      )),
    };

    let offset_value_parts: Vec<&str> = offset_value.split("..").collect();

    // Set start offset
    {
      let start_offset = sub_context.borrow().offset.start;
      sub_context.borrow_mut().offset.start =
        parse_offset(start_offset, start_offset, offset_value_parts[0]);
    }

    // Set end offset (if specified)
    if offset_value_parts.len() > 1 {
      let start_offset = sub_context.borrow().offset.start;
      let end_offset = sub_context.borrow().offset.end;
      sub_context.borrow_mut().offset.end =
        parse_offset(start_offset, end_offset, offset_value_parts[1]);
    }

    let start_offset = sub_context.borrow().offset.start;
    let end_offset = sub_context.borrow().offset.end;

    match &self.pattern {
      Some(matcher) => matcher.borrow().exec(sub_context.clone()),
      None => Ok(MatcherSuccess::Token(PinToken::new_with_matched_range(
        &sub_context.borrow().parser,
        self.get_name().to_string(),
        SourceRange::new(start_offset, end_offset),
        SourceRange::new(start_offset, end_offset),
      ))),
    }
  }

  fn is_consuming(&self) -> bool {
    false
  }

  fn has_custom_name(&self) -> bool {
    false
  }

  fn get_name(&self) -> &str {
    "Pin"
  }

  fn set_name(&mut self, _: &str) {
    panic!("Can not set `name` on a `Pin` matcher");
  }

  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>> {
    match self.pattern {
      Some(ref matcher) => Some(vec![matcher.clone()]),
      None => None,
    }
  }

  fn add_pattern(&mut self, _: MatcherRef<'a>) {
    panic!("Can not add a pattern to a `Equals` matcher");
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Pin {
  ($offset:literal; $arg:expr) => {
    $crate::matchers::pin::PinPattern::new($offset, Some($arg))
  };

  ($offset:expr; $arg:expr) => {
    $crate::matchers::pin::PinPattern::new($offset, Some($arg))
  };

  ($arg:expr) => {
    $crate::matchers::pin::PinPattern::new("", Some($arg))
  };

  ($offset:literal) => {
    $crate::matchers::pin::PinPattern::new($offset, None)
  };

  () => {
    $crate::matchers::pin::PinPattern::new("", None)
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Discard, Equals, Fetch, Matches, Program, Store,
  };

  #[test]
  fn it_shouldnt_update_context_offset() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Pin!("8"; Equals!("1234"));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(8, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(8, 12));
      assert_eq!(token.get_value(), "1234");

      // Offset should not have been updated with a Pin
      assert_eq!(parser_context.borrow().offset.start, 0);
      assert_eq!(parser_context.borrow().offset.end, 12);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_be_able_to_store_a_pin() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Store!("stored_offset"; Pin!()),
      Equals!("Testing"),
      Pin!(Fetch!("stored_offset.range"); Equals!("Testing")),
      Discard!(Matches!(r"\s+")),
      Matches!(r"\d+"),
    );

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), "Testing 1234");
      assert_eq!(token.get_children().len(), 3);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Equals");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Equals");
      assert_eq!(*second.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*second.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(second.get_value(), "Testing");

      let third = token.get_children()[2].borrow();
      assert_eq!(third.get_name(), "Matches");
      assert_eq!(*third.get_captured_range(), SourceRange::new(8, 12));
      assert_eq!(*third.get_matched_range(), SourceRange::new(8, 12));
      assert_eq!(third.get_value(), "1234");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_be_able_to_specify_a_relative_offset1() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Pin!("+8"; Equals!("1234"));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(8, 12));
      assert_eq!(*token.get_matched_range(), SourceRange::new(8, 12));
      assert_eq!(token.get_value(), "1234");

      // Offset should not have been updated with a Pin
      assert_eq!(parser_context.borrow().offset.start, 0);
      assert_eq!(parser_context.borrow().offset.end, 12);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_be_able_to_specify_a_relative_offset2() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Equals!("Testing"), Pin!("-7"; Equals!("Testing")));

    if let Ok(MatcherSuccess::Token(token)) =
      ParserContext::tokenize(parser_context.clone(), matcher)
    {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "Equals");
      assert_eq!(*first.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*first.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(first.get_value(), "Testing");

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Equals");
      assert_eq!(*second.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(*second.get_matched_range(), SourceRange::new(0, 7));
      assert_eq!(second.get_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_should_fail_with_too_tight_a_range() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Pin!("0..4"; Equals!("Testing"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context.clone(), matcher)
    )
  }
}
