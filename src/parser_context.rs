use regex::Regex;

use super::parser::Parser;
use super::source_range::SourceRange;

#[derive(Clone, Copy)]
pub struct ParserContext<'a> {
  pub offset: SourceRange,
  pub parser: &'a Parser,
}

impl<'a> ParserContext<'a> {
  pub fn new<'b>(parser: &'b Parser) -> ParserContext<'b> {
    ParserContext {
      offset: SourceRange::new(0, parser.source.len()),
      parser,
    }
  }

  pub fn new_with_offset<'b>(parser: &'b Parser, offset: SourceRange) -> ParserContext<'b> {
    ParserContext { offset, parser }
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
    let chunk = &self.parser.source[self.offset.start..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(self.offset.clone_with_len(pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_regexp(&self, pattern: &Regex) -> Option<SourceRange> {
    let chunk = &self.parser.source[self.offset.start..self.offset.end];

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
