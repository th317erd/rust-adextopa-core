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

      fn get_value_range(&self) -> &crate::source_range::SourceRange {
        &self.value_range
      }

      fn get_value_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
        &mut self.value_range
      }

      fn set_value_range(&mut self, range: crate::source_range::SourceRange) {
        self.value_range = range;
      }

      fn get_raw_range(&self) -> &crate::source_range::SourceRange {
        &self.raw_range
      }

      fn get_raw_range_mut(&mut self) -> &mut crate::source_range::SourceRange {
        &mut self.raw_range
      }

      fn set_raw_range(&mut self, range: crate::source_range::SourceRange) {
        self.raw_range = range;
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

      fn value(&self) -> String {
        // Value override via attribute
        match self.get_attribute("__value") {
          Some(value) => {
            return value.clone();
          },
          None => {}
        }

        self.value_range.to_string(&self.parser)
      }

      fn raw_value(&self) -> String {
        // Value override via attribute
        match self.get_attribute("__raw_value") {
          Some(value) => {
            return value.clone();
          },
          None => {}
        }

        self.raw_range.to_string(&self.parser)
      }

      fn get_attributes<'b>(&'b self) -> &'b std::collections::HashMap<String, String> {
        &self.attributes
      }

      fn get_attribute<'b>(&'b self, name: &str) -> Option<&'b String> {
        self.attributes.get(&name.to_string())
      }

      fn set_attribute(&mut self, name: &str, value: &str) -> Option<String> {
        self.attributes.insert(name.to_string(), value.to_string())
      }
    }
  };

  // Return the generated impl
  TokenStream::from(expanded)
}
