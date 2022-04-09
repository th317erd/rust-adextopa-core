use super::token::TokenRef;
use crate::parser_context::ParserContextRef;
use crate::scope::VariableType;
use crate::scope_context::ScopeContextRef;
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

pub type MatcherRef = Rc<RefCell<Box<dyn Matcher>>>;

impl std::fmt::Debug for dyn Matcher {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.to_string())
  }
}

pub trait Matcher {
  fn before_exec(&self, this_matcher: MatcherRef, _: ParserContextRef, scope: ScopeContextRef) {
    if self.has_custom_name() {
      scope
        .borrow_mut()
        .set(self.get_name(), VariableType::Matcher(this_matcher.clone()));
    }
  }

  fn after_exec(&self, _: MatcherRef, _: ParserContextRef, _: ScopeContextRef) {}

  fn exec(
    &self,
    this_matcher: MatcherRef,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure>;
  fn get_name(&self) -> &str;
  fn set_name(&mut self, name: &str);
  fn add_pattern(&mut self, pattern: MatcherRef);
  fn get_children(&self) -> Option<Vec<MatcherRef>>;
  fn to_string(&self) -> String;

  fn set_child(&mut self, _: usize, _: MatcherRef) {
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
