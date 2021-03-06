extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

// use crate::adextopa_core::parser::Parser;
use proc_macro::TokenStream;

#[proc_macro_derive(Token)]
pub fn token_derive(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  // Build the impl
  let name = &input.ident;
  let expanded = quote! {
    impl Token for #name {
      fn get_parser(&self) -> crate::parser::ParserRef {
        self.parser.clone()
      }

      fn get_captured_range(&self) -> &crate::source_range::SourceRange {
        &self.captured_range
      }

      fn get_captured_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
        &mut self.captured_range
      }

      fn set_captured_range(&mut self, range: crate::source_range::SourceRange) {
        self.captured_range = range;
        self.captured_value = range.to_string(&self.parser);
      }

      fn get_matched_range(&self) -> &crate::source_range::SourceRange {
        &self.matched_range
      }

      fn get_matched_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
        &mut self.matched_range
      }

      fn set_matched_range(&mut self, range: crate::source_range::SourceRange) {
        self.matched_range = range;
        self.matched_value = range.to_string(&self.parser);
      }

      fn get_name(&self) -> &String {
        &self.name
      }

      fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
      }

      fn get_parent(&self) -> Option<crate::token::TokenRef> {
        match self.parent {
          Some(ref token_ref) => Some(token_ref.clone()),
          None => None,
        }
      }

      fn set_parent(&mut self, token: Option<crate::token::TokenRef>) {
        self.parent = token;
      }

      fn get_children<'b>(&'b self) -> &'b Vec<crate::token::TokenRef> {
        &self.children
      }

      fn get_children_mut<'b>(&'b mut self) -> &'b mut Vec<crate::token::TokenRef> {
        &mut self.children
      }

      fn set_children(&mut self, children: Vec<crate::token::TokenRef>) {
        self.children = children;
      }

      fn add_child(&mut self, child: crate::token::TokenRef) {
        self.children.push(child.clone())
      }

      fn get_value(&self) -> &String {
        // Value override via attribute
        match self.get_attribute("__value") {
          Some(value) => {
            return value;
          }
          None => {}
        }

        match &self.value {
          Some(ref value) => value,
          None => self.get_captured_value(),
        }
      }

      fn set_value(&mut self, value: &str) {
        if value == "" {
          self.value = None;
        } else {
          self.value = Some(value.to_string());
        }
      }

      fn get_captured_value(&self) -> &String {
        &self.captured_value
      }

      fn set_captured_value(&mut self, value: &str) {
        self.captured_value = value.to_string();
      }

      fn get_matched_value(&self) -> &String {
        &self.matched_value
      }

      fn set_matched_value(&mut self, value: &str) {
        self.matched_value = value.to_string();
      }

      fn get_attributes<'b>(&'b self) -> &'b std::collections::HashMap<String, String> {
        &self.attributes
      }

      fn get_attribute<'b>(&'b self, name: &str) -> Option<&'b String> {
        self.attributes.get(&name.to_string())
      }

      fn attribute_equals<'b>(&'b self, name: &str, value: &str) -> bool {
        match self.attributes.get(&name.to_string()) {
          Some(v) => (v == value),
          None => false,
        }
      }

      fn set_attribute(&mut self, name: &str, value: &str) -> Option<String> {
        self.attributes.insert(name.to_string(), value.to_string())
      }

      fn get_flags(&mut self) -> TokenType {
        self.flags
      }

      fn set_flags(&mut self, flags: TokenType) {
        self.flags = flags;
      }

      fn enable_flags(&mut self, flags: TokenType) {
        self.flags = self.flags | flags;
      }

      fn disable_flags(&mut self, flags: TokenType) {
        self.flags = self.flags & !flags;
      }

      fn flags_enabled(&self, flags: TokenType) -> bool {
        (self.flags & flags) != 0
      }
    }
  };

  // Return the generated impl
  TokenStream::from(expanded)
}
