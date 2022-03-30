pub mod matcher;
pub mod matchers;
pub mod parser;
pub mod parser_context;
pub mod scope;
pub mod scope_context;
pub mod script;
pub mod source_range;
pub mod token;
pub mod token_visitor;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}
