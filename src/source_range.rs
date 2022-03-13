use substring::Substring;

use crate::parser::Parser;

use super::parser::ParserRef;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourceRange {
  pub start: usize,
  pub end: usize,
}

impl SourceRange {
  pub fn new(start: usize, end: usize) -> Self {
    Self { start, end }
  }

  pub fn new_blank() -> Self {
    Self { start: 0, end: 0 }
  }

  pub fn to_string<'a>(&self, parser: &ParserRef) -> String {
    parser
      .borrow()
      .source
      .substring(self.start, self.end)
      .to_string()
  }

  pub fn clone_with_len(&self, len: usize) -> Self {
    Self {
      start: self.start,
      end: self.start + len,
    }
  }
}

impl std::fmt::Display for SourceRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{}-{}]", self.start, self.end)
  }
}

mod tests {
  use super::*;
  use crate::parser::Parser;

  #[test]
  fn it_can_be_cloned() {
    let sr1 = SourceRange::new(0, 10);
    let sr2 = sr1.clone();

    assert!(!std::ptr::eq(&sr1, &sr2));
    assert_eq!(sr1.start, sr2.start);
    assert_eq!(sr1.end, sr2.end);
  }

  #[test]
  fn it_can_be_displayed() {
    let sr1 = SourceRange::new(0, 10);

    assert_eq!(format!("{}", sr1), "[0-10]");
  }

  #[test]
  fn it_can_get_range_slice() {
    let parser = Parser::new("Hello world!");
    let sr1 = SourceRange::new(0, 5);

    assert_eq!(sr1.to_string(&parser), "Hello");
  }
}
