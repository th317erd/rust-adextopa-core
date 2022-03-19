use regex::Regex;

use super::source_range::SourceRange;
use crate::parser::ParserRef;

pub type ParserContextRef<'a> = std::rc::Rc<std::cell::RefCell<ParserContext<'a>>>;

#[derive(Clone)]
pub struct ParserContext<'a> {
  debug_mode: usize,
  pub offset: SourceRange,
  pub parser: ParserRef,
  pub name: &'a str,
}

impl<'a> ParserContext<'a> {
  pub fn new<'b>(parser: &ParserRef, name: &'b str) -> ParserContextRef<'b> {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      offset: SourceRange::new(0, parser.borrow().source.len()),
      parser: parser.clone(),
      debug_mode: 0,
      name,
    }))
  }

  pub fn new_with_offset<'b>(
    parser: &ParserRef,
    offset: SourceRange,
    name: &'b str,
  ) -> ParserContextRef<'b> {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      offset,
      parser: parser.clone(),
      debug_mode: 0,
      name,
    }))
  }

  pub fn clone_with_name<'b>(&'b self, name: &'b str) -> ParserContextRef<'b> {
    let mut c = self.clone();
    c.name = name;
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
}
