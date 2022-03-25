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
  fn set_name(&mut self, name: &str);
  fn add_pattern(&mut self, pattern: MatcherRef<'a>);
  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>>;

  fn set_child(&mut self, _: usize, _: MatcherRef<'a>) {
    panic!(
      "Can not call `set_child` on a `{}` matcher: Operation not supported",
      self.get_name()
    );
  }

  fn has_custom_name(&self) -> bool {
    false
  }

  fn swap_with_reference_name(&self) -> Option<&'a str> {
    None
  }

  fn is_consuming(&self) -> bool {
    true
  }
}
