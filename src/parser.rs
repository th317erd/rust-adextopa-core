use std::{fs, io::Error};

pub type ParserRef = std::rc::Rc<std::cell::RefCell<Parser>>;

#[allow(unused)]
#[derive(Debug)]
pub struct Parser {
  pub(crate) source: String,
  pub(crate) filename: String,
}

impl Parser {
  pub fn new(source: &str) -> ParserRef {
    std::rc::Rc::new(std::cell::RefCell::new(Self {
      source: source.to_string(),
      filename: String::from(""),
    }))
  }

  pub fn new_with_file_name(source: &str, filename: &str) -> ParserRef {
    std::rc::Rc::new(std::cell::RefCell::new(Self {
      source: source.to_string(),
      filename: filename.to_string(),
    }))
  }

  pub fn new_from_file(filename: &str) -> Result<ParserRef, Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(std::rc::Rc::new(std::cell::RefCell::new(Self {
      source: contents,
      filename: String::from(filename),
    })))
  }

  pub fn tokenize() {}
}
