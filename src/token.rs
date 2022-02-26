use std::cell::RefCell;
use std::rc::Rc;

use super::parser::Parser;
use super::source_range::SourceRange;

// Need + 'a or 'static is implied
pub type TokenRefInner<'a> = dyn Token<'a> + 'a;
pub type TokenRef<'a> = Rc<RefCell<Box<TokenRefInner<'a>>>>;

impl<'a> core::fmt::Debug for Box<TokenRefInner<'a>> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Box")
      .field(&self.get_name())
      .field(&self.get_value_range())
      .field(&self.get_raw_range())
      .finish()
  }
}

impl<'a> PartialEq for Box<TokenRefInner<'a>> {
  fn eq(&self, other: &Self) -> bool {
    *self.get_name() == *other.get_name()
      && *self.get_value_range() == *other.get_value_range()
      && *self.get_raw_range() == *other.get_raw_range()
  }
}

// Token<'a> is required because name: &'a str is required (name lives as long as the underlying struct)
pub trait Token<'a> {
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
  fn to_string<'b>(&self, parser: &'b Parser) -> &'b str;
  fn value<'b>(&self, parser: &'b Parser) -> &'b str;
  fn raw_value<'b>(&self, parser: &'b Parser) -> &'b str;
}
