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

impl<'a> std::fmt::Debug for dyn Matcher<'a> + 'a {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.to_string())
  }
}

// trait CustomDebug: std::fmt::Debug {}

// impl<'a> CustomDebug for MatcherRef<'a> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     f.debug_struct("Rc")
//       .field("ptr", &self.ptr)
//       .field("phantom", &self.phantom)
//       .finish()
//   }
// }

pub trait Matcher<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure>;
  fn get_name(&self) -> &str;
  fn set_name(&mut self, name: &str);
  fn add_pattern(&mut self, pattern: MatcherRef<'a>);
  fn get_children(&self) -> Option<Vec<MatcherRef<'a>>>;
  fn to_string(&self) -> String;

  fn set_scope(&mut self, scope: Option<&str>) {
    // NO-OP
  }

  fn set_child(&mut self, _: usize, _: MatcherRef<'a>) {
    panic!(
      "Can not call `set_child` on a `{}` matcher: Operation not supported",
      self.get_name()
    );
  }

  fn has_custom_name(&self) -> bool {
    false
  }

  fn is_consuming(&self) -> bool {
    true
  }
}
