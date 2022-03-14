use regex::Regex;

use super::source_range::SourceRange;
use crate::{parser::ParserRef, source_range};

pub type ParserContextRef = std::rc::Rc<std::cell::RefCell<ParserContext>>;

#[derive(Clone)]
pub struct ParserContext {
  pub offset: SourceRange,
  pub parser: ParserRef,
}

impl ParserContext {
  pub fn new(parser: &ParserRef) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      offset: SourceRange::new(0, parser.borrow().source.len()),
      parser: parser.clone(),
    }))
  }

  pub fn new_with_offset(parser: &ParserRef, offset: SourceRange) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      offset,
      parser: parser.clone(),
    }))
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
    let chunk = &self.parser.borrow().source[self.offset.start..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(self.offset.clone_with_len(pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_str_at_offset(&self, pattern: &str, offset: usize) -> Option<SourceRange> {
    if offset >= self.offset.end {
      if pattern.len() == 0 {
        return Some(SourceRange::new(offset, offset));
      } else {
        return None;
      }
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
}
