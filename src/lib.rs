pub mod matcher;
pub mod matchers;
pub mod parser;
pub mod parser_context;
pub mod source_range;
pub mod token;
pub(crate) mod utils;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}
