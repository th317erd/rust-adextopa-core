use crate::source_range::SourceRange;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
  pub message: String,
  pub range: Option<SourceRange>,
}

impl ParseError {
  pub fn new(message: &str) -> Self {
    Self {
      message: message.to_string(),
      range: None,
    }
  }

  pub fn new_with_range(message: &str, range: SourceRange) -> Self {
    Self {
      message: message.to_string(),
      range: Some(range),
    }
  }
}
