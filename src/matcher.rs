use super::token::TokenRef;
use crate::parser_context::ParserContextRef;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum MatcherSuccess {
  Token(TokenRef),
  ExtractChildren(TokenRef),
  Skip(isize),
  Break((String, Box<MatcherSuccess>)),
  Continue((String, Box<MatcherSuccess>)),
  None,
  Stop,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MatcherFailure {
  Fail,
  Error(String),
}

pub type MatcherRef<'a> = Rc<RefCell<Box<dyn Matcher<'a> + 'a>>>;

pub trait Matcher<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure>;
  fn get_name(&self) -> &str;
  fn set_name(&mut self, name: &'a str);
  fn add_pattern(&mut self, pattern: MatcherRef<'a>);
  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>>;
}
