use super::source_range::SourceRange;
use crate::{
  matcher::{MatcherFailure, MatcherRef, MatcherSuccess},
  parser::ParserRef,
  scope::VariableType,
  scope_context::{ScopeContext, ScopeContextRef},
};
use regex::Regex;
use std::{cell::RefCell, rc::Rc};

pub type ParserContextRef = Rc<RefCell<ParserContext>>;

#[derive(Clone)]
pub struct ParserContext {
  pub(crate) debug_mode: usize,
  pub(crate) scope: ScopeContextRef,
  pub offset: SourceRange,
  pub parser: ParserRef,
  pub name: String,
}

impl ParserContext {
  pub fn new(parser: &ParserRef, name: &str) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      scope: ScopeContext::new(),
      offset: SourceRange::new(0, parser.borrow().source.len()),
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn new_with_offset(parser: &ParserRef, offset: SourceRange, name: &str) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      scope: ScopeContext::new(),
      offset,
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn clone_with_name(&self, name: &str) -> ParserContextRef {
    let mut c = self.clone();
    c.name = name.to_string();
    std::rc::Rc::new(std::cell::RefCell::new(c))
  }

  pub fn is_debug_mode(&self) -> bool {
    self.debug_mode > 0
  }

  pub fn debug_mode_level(&self) -> usize {
    self.debug_mode
  }

  pub fn set_debug_mode(&mut self, value: usize) {
    self.debug_mode = value;
  }

  pub fn set_start(&mut self, start: usize) {
    self.offset.start = start;
  }

  pub fn set_end(&mut self, end: usize) {
    self.offset.end = end;
  }

  pub fn set_offset(&mut self, range: SourceRange) {
    self.offset = range;
  }

  pub fn get_scope_variable(&self, name: &str) -> Option<VariableType> {
    self.scope.borrow().get(name)
  }

  pub fn set_scope_variable(&mut self, name: &str, value: VariableType) -> Option<VariableType> {
    self.scope.borrow_mut().set(name, value)
  }

  pub fn matches_str(&self, pattern: &str) -> Option<SourceRange> {
    if pattern.len() == 0 {
      return None;
    }

    let chunk = &self.parser.borrow().source[self.offset.start..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(self.offset.clone_with_len(pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_str_at_offset(&self, pattern: &str, offset: usize) -> Option<SourceRange> {
    if offset >= self.offset.end || pattern.len() == 0 {
      return None;
    }

    let chunk = &self.parser.borrow().source[offset..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(SourceRange::new(offset, offset + pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_regexp(&self, pattern: &Regex) -> Option<SourceRange> {
    let chunk = &self.parser.borrow().source[self.offset.start..self.offset.end];

    match pattern.find(chunk) {
      Some(m) => {
        let start = m.start() + self.offset.start;
        let end = m.end() + self.offset.start;

        if start != self.offset.start {
          None
        } else {
          Some(SourceRange::new(start, end))
        }
      }
      None => None,
    }
  }

  pub fn debug_range(&self, max_len: usize) -> String {
    let parser = self.parser.borrow();
    let mut end_offset = self.offset.start + max_len;

    if end_offset > self.offset.end {
      end_offset = self.offset.end;
    }

    parser.source[self.offset.start..end_offset].to_string()
  }

  pub fn capture_matcher_references(&self, matcher: MatcherRef) {
    let m = matcher.borrow();

    if m.has_custom_name() {
      let name = m.get_name();

      if self.debug_mode > 1 {
        println!("Registering matcher `{}`", name);
      }

      self
        .scope
        .borrow_mut()
        .set(name, VariableType::Matcher(matcher.clone()));
    }

    match m.get_children() {
      Some(children) => {
        for child in children {
          self.capture_matcher_references(child.clone());
        }
      }
      None => {}
    }
  }

  pub fn register_matchers(&self, matchers: Vec<MatcherRef>) {
    for matcher in matchers {
      self.capture_matcher_references(matcher.clone());
    }
  }

  pub fn register_matcher(&self, matcher: MatcherRef) {
    self.capture_matcher_references(matcher.clone());
  }

  pub fn get_registered_matcher(&self, name: &str) -> Option<MatcherRef> {
    match self.scope.borrow().get(name) {
      Some(VariableType::Matcher(matcher)) => Some(matcher.clone()),
      _ => None,
    }
  }

  pub fn tokenize(
    context: ParserContextRef,
    matcher: MatcherRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    context.borrow().capture_matcher_references(matcher.clone());

    let scope = context.borrow().scope.clone();
    matcher.borrow().exec(context.clone(), scope)
  }
}
