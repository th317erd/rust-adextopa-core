use std::cell::RefCell;
use std::rc::Rc;

use regex::Regex;

use crate::parser::ParserRef;

use super::source_range::SourceRange;

// Need + 'a or 'static is implied
pub type TokenRefInner = dyn Token;
pub type TokenRef = Rc<RefCell<Box<TokenRefInner>>>;

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

impl core::fmt::Debug for TokenRefInner {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let tabs = get_tab_depth_for_debug(Box::new(self));
    let tab_minus_one = &tabs[2..];

    let result = write!(
      f,
      "{}{}(\n{}ValueRange({}, {}),\n{}RawRange({}, {}),\n{}Value({}),\n{}RawValue({}),\n{}Parent({}),\n{}Attributes {:?}\n{}Children [",
      tab_minus_one,
      self.get_name(),
      tabs,
      self.get_captured_range().start,
      self.get_captured_range().end,
      tabs,
      self.get_matched_range().start,
      self.get_matched_range().end,
      tabs,
      self.get_captured_value(),
      tabs,
      self.get_matched_value(),
      tabs,
      get_parent_path_for_debug(Box::new(self)),
      tabs,
      self.get_attributes(),
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

impl std::fmt::Display for TokenRefInner {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl PartialEq for TokenRefInner {
  fn eq(&self, other: &Self) -> bool {
    *self.get_name() == *other.get_name()
      && *self.get_captured_range() == *other.get_captured_range()
      && *self.get_matched_range() == *other.get_matched_range()
  }
}

// Token<'a> is required because name: &'a str is required (name lives as long as the underlying struct)
pub trait Token {
  fn get_parser(&self) -> crate::parser::ParserRef;
  fn get_captured_range(&self) -> &crate::source_range::SourceRange;
  fn get_captured_range_mut(&mut self) -> &mut crate::source_range::SourceRange;
  fn set_captured_range(&mut self, range: crate::source_range::SourceRange);
  fn get_matched_range(&self) -> &crate::source_range::SourceRange;
  fn get_matched_range_mut(&mut self) -> &mut crate::source_range::SourceRange;
  fn set_matched_range(&mut self, range: crate::source_range::SourceRange);
  fn get_name(&self) -> &String;
  fn set_name(&mut self, name: &str);
  fn get_parent(&self) -> Option<crate::token::TokenRef>;
  fn set_parent(&mut self, token: Option<crate::token::TokenRef>);
  fn get_children<'b>(&'b self) -> &'b Vec<crate::token::TokenRef>;
  fn get_children_mut<'b>(&'b mut self) -> &'b mut Vec<crate::token::TokenRef>;
  fn set_children(&mut self, children: Vec<crate::token::TokenRef>);
  fn get_captured_value(&self) -> &String;
  fn set_captured_value(&mut self, value: &str);
  fn get_matched_value(&self) -> &String;
  fn set_matched_value(&mut self, value: &str);
  fn get_attributes<'b>(&'b self) -> &'b std::collections::HashMap<String, String>;
  fn get_attribute<'b>(&'b self, name: &str) -> Option<&'b String>;
  fn attribute_equals<'b>(&'b self, name: &str, value: &str) -> bool;
  fn set_attribute(&mut self, name: &str, value: &str) -> Option<String>;

  fn has_attribute<'b>(&'b self, name: &str) -> bool {
    match self.get_attribute(name) {
      Some(_) => true,
      None => false,
    }
  }

  fn find_child(&self, needle: &str) -> Option<crate::token::TokenRef> {
    let children = self.get_children();
    if children.len() == 0 {
      return None;
    }

    for child in children {
      let _child = child.borrow();
      if _child.get_name() == needle {
        return Some(child.clone());
      }
    }

    None
  }

  fn find_child_fuzzy(&self, regex: &Regex) -> Option<crate::token::TokenRef> {
    let children = self.get_children();
    if children.len() == 0 {
      return None;
    }

    for child in children {
      let _child = child.borrow();
      if regex.is_match(_child.get_name()) {
        return Some(child.clone());
      }
    }

    None
  }

  fn has_child(&self, needle: &str) -> bool {
    match self.find_child(needle) {
      Some(_) => true,
      None => false,
    }
  }

  fn should_discard(&self) -> bool {
    false
  }
}

#[derive(adextopa_macros::Token)]
pub struct StandardToken {
  parser: ParserRef,
  pub captured_range: SourceRange,
  pub matched_range: SourceRange,
  pub name: String,
  pub captured_value: String,
  pub matched_value: String,
  pub parent: Option<TokenRef>,
  pub children: Vec<TokenRef>,
  pub attributes: std::collections::HashMap<String, String>,
}

impl StandardToken {
  pub fn new(parser: &ParserRef, name: String, captured_range: SourceRange) -> TokenRef {
    Rc::new(RefCell::new(Box::new(StandardToken {
      parser: parser.clone(),
      captured_range,
      matched_range: captured_range.clone(),
      name,
      captured_value: captured_range.to_string(&parser),
      matched_value: captured_range.to_string(&parser),
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }

  pub fn new_with_matched_range(
    parser: &ParserRef,
    name: String,
    captured_range: SourceRange,
    matched_range: SourceRange,
  ) -> TokenRef {
    Rc::new(RefCell::new(Box::new(StandardToken {
      parser: parser.clone(),
      captured_range,
      matched_range,
      name,
      captured_value: captured_range.to_string(&parser),
      matched_value: matched_range.to_string(&parser),
      parent: None,
      children: Vec::new(),
      attributes: std::collections::HashMap::new(),
    })))
  }
}
