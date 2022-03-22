use super::source_range::SourceRange;
use crate::{
  matcher::{MatcherFailure, MatcherRef, MatcherSuccess},
  parser::ParserRef,
  token::TokenRef,
};
use regex::Regex;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(PartialEq, Debug)]
pub enum VariableType {
  Token(TokenRef),
  String(String),
}

pub type ParserContextRef<'a> = Rc<RefCell<ParserContext<'a>>>;

#[derive(Clone)]
pub struct ParserContext<'a> {
  debug_mode: usize,
  matcher_reference_map: Rc<RefCell<HashMap<String, MatcherRef<'a>>>>,
  pub variable_context: Rc<RefCell<HashMap<String, VariableType>>>,
  pub offset: SourceRange,
  pub parser: ParserRef,
  pub name: String,
}

impl<'a> ParserContext<'a> {
  pub fn new<'b>(parser: &ParserRef, name: &'b str) -> ParserContextRef<'b> {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      matcher_reference_map: Rc::new(RefCell::new(HashMap::new())),
      variable_context: Rc::new(RefCell::new(HashMap::new())),
      offset: SourceRange::new(0, parser.borrow().source.len()),
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn new_with_offset<'b>(
    parser: &ParserRef,
    offset: SourceRange,
    name: &'b str,
  ) -> ParserContextRef<'b> {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      matcher_reference_map: Rc::new(RefCell::new(HashMap::new())),
      variable_context: Rc::new(RefCell::new(HashMap::new())),
      offset,
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn clone_with_name(&self, name: &str) -> ParserContextRef<'a> {
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

  pub fn get_variable(&self, name: &str) -> Option<VariableType> {
    match self.variable_context.borrow().get(name) {
      Some(VariableType::Token(value)) => Some(VariableType::Token(value.clone())),
      Some(VariableType::String(value)) => Some(VariableType::String(value.clone())),
      None => None,
    }
  }

  pub fn set_variable(&mut self, name: String, value: VariableType) -> Option<VariableType> {
    self.variable_context.borrow_mut().insert(name, value)
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

  fn capture_matcher_references(&self, matcher: MatcherRef<'a>) {
    let m = matcher.borrow();
    if m.has_custom_name() {
      let name = m.get_name();
      self
        .matcher_reference_map
        .borrow_mut()
        .insert(name.to_string(), matcher.clone());
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

  fn substitute_matcher_references(&self, matcher: MatcherRef<'a>) {
    let children = matcher.borrow().get_children();

    match children {
      Some(children) => {
        let mut index: usize = 0;

        for child in children {
          let ref_name = child.borrow().swap_with_reference_name();
          match ref_name {
            Some(name) => {
              let ref_map = self.matcher_reference_map.borrow();
              let reference = ref_map.get(&name.to_string());
              match reference {
                Some(matcher_ref) => matcher.borrow_mut().set_child(index, matcher_ref.clone()),
                None => {
                  panic!("Unable to find pattern reference named `{}`", name);
                }
              }
            }
            None => {
              self.substitute_matcher_references(child.clone());
            }
          }

          index += 1;
        }
      }
      None => {}
    }
  }

  pub fn register_matchers(&self, matchers: Vec<MatcherRef<'a>>) {
    for matcher in matchers {
      self.capture_matcher_references(matcher.clone());
    }
  }

  pub fn tokenize(
    context: ParserContextRef<'a>,
    matcher: MatcherRef<'a>,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    context.borrow().capture_matcher_references(matcher.clone());
    context
      .borrow()
      .substitute_matcher_references(matcher.clone());

    matcher.borrow().exec(context.clone())
  }
}
