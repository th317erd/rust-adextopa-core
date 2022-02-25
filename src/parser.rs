use std::{fs, io::Error};

use crate::{parser_context::ParserContext, source_range::SourceRange};

pub struct Parser {
  pub(crate) source: String,
  pub(crate) filename: String,
}

impl Parser {
  pub fn new(source: &str) -> Self {
    Self {
      source: source.to_string(),
      filename: String::from(""),
    }
  }

  pub fn new_with_file_name(source: &str, filename: &str) -> Self {
    Self {
      source: source.to_string(),
      filename: filename.to_string(),
    }
  }

  pub fn new_from_file(filename: &str) -> Result<Self, Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(Self {
      source: contents,
      filename: String::from(filename),
    })
  }

  pub fn tokenize() {}
}
