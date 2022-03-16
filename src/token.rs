use std::cell::RefCell;
use std::rc::Rc;

use crate::parser::ParserRef;

use super::source_range::SourceRange;

// Need + 'a or 'static is implied
pub type TokenRefInner<'a> = dyn Token<'a> + 'a;
pub type TokenRef<'a> = Rc<RefCell<Box<TokenRefInner<'a>>>>;

fn get_parent_path_for_debug<'a>(token: Box<&dyn Token>) -> String {
  match token.get_parent() {
    Some(token) => {
      let token = RefCell::borrow(&token);
      return token.get_name().to_string();
    }
    None => {
      return "None".to_string();
    }
  }
}

fn get_tab_depth_for_debug(token: Box<&dyn Token>) -> String {
  let mut parts = Vec::<String>::new();
  let mut parent = token.get_parent();

  parts.push("  ".to_string());

  loop {
    match parent {
      Some(token) => {
        let token = RefCell::borrow(&token);
        parts.push("  ".to_string());
        parent = token.get_parent();
      }
      None => {
        break;
      }
    }
  }

  parts.join("")
}

impl<'a> core::fmt::Debug for TokenRefInner<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let tabs = get_tab_depth_for_debug(Box::new(self));
    let tab_minus_one = &tabs[2..];

    let result = write!(
      f,
      "{}{}(\n{}ValueRange({}, {}),\n{}RawRange({}, {}),\n{}Value({}),\n{}RawValue({}),\n{}Parent({}),\n{}Children [",
      tab_minus_one,
      self.get_name(),
      tabs,
      self.get_value_range().start,
      self.get_value_range().end,
      tabs,
      self.get_raw_range().start,
      self.get_raw_range().end,
      tabs,
      self.value(),
      tabs,
      self.raw_value(),
      tabs,
      get_parent_path_for_debug(Box::new(self)),
      tabs,
    );

    if let Err(error) = result {
      return Err(error);
    }

    let next_level_tabs = format!("{}  ", tabs);

    let children = self.get_children();
    for child in children {
      let child_str = format!("\n{:?}", std::cell::RefCell::borrow(&child));

      if let Err(error) = f.write_str(child_str.as_str().replace("\n", "\n  ").as_str()) {
        return Err(error);
      }

      if let Err(error) = f.write_str(format!("\n{}),\n{}", next_level_tabs, tabs).as_str()) {
        return Err(error);
      }
    }

    f.write_str("],")
  }
}

impl<'a> std::fmt::Display for TokenRefInner<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl<'a> PartialEq for TokenRefInner<'a> {
  fn eq(&self, other: &Self) -> bool {
    *self.get_name() == *other.get_name()
      && *self.get_value_range() == *other.get_value_range()
      && *self.get_raw_range() == *other.get_raw_range()
  }
}

// Token<'a> is required because name: &'a str is required (name lives as long as the underlying struct)
pub trait Token<'a> {
  fn get_parser(&self) -> ParserRef;
  fn get_value_range(&self) -> &SourceRange;
  fn get_value_range_mut(&mut self) -> &mut SourceRange;
  fn set_value_range(&mut self, range: SourceRange);
  fn get_raw_range(&self) -> &SourceRange;
  fn get_raw_range_mut(&mut self) -> &mut SourceRange;
  fn set_raw_range(&mut self, range: SourceRange);
  fn get_name(&self) -> &str;
  fn set_name(&mut self, name: &'a str);
  fn get_parent(&self) -> Option<TokenRef<'a>>;
  fn set_parent(&mut self, token: Option<crate::token::TokenRef<'a>>);
  fn get_children<'b>(&'b self) -> &'b Vec<crate::token::TokenRef<'a>>;
  fn get_children_mut<'b>(&'b mut self) -> &'b mut Vec<crate::token::TokenRef<'a>>;
  fn set_children(&mut self, children: Vec<crate::token::TokenRef<'a>>);
  fn value(&self) -> String;
  fn raw_value(&self) -> String;
  fn get_attributes<'b>(&'b self) -> &'b std::collections::HashMap<String, String>;
  fn get_attribute<'b>(&'b self, name: String) -> Option<&'b String>;
  fn set_attribute(&mut self, name: String, value: String) -> Option<String>;
}

#[derive(adextopa_macros::Token)]
pub struct StandardToken<'a> {
  parser: ParserRef,
  pub value_range: SourceRange,
  pub raw_range: SourceRange,
  pub name: &'a str,
  pub parent: Option<TokenRef<'a>>,
  pub children: Vec<TokenRef<'a>>,
  pub attributes: std::collections::HashMap<String, String>,
}

impl<'a> StandardToken<'a> {
  pub fn new(parser: &ParserRef, name: &'a str, value_range: SourceRange) -> TokenRef<'a> {
    Rc::new(RefCell::new(Box::new(StandardToken {
      parser: parser.clone(),
      value_range,
      raw_range: value_range.clone(),
      name,
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }

  pub fn new_with_raw_range(
    parser: &ParserRef,
    name: &'a str,
    value_range: SourceRange,
    raw_range: SourceRange,
  ) -> TokenRef<'a> {
    Rc::new(RefCell::new(Box::new(StandardToken {
      parser: parser.clone(),
      value_range,
      raw_range,
      name,
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }
}
