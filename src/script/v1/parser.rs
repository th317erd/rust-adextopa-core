use crate::{
  matcher::{MatcherRef, MatcherSuccess},
  parser::Parser,
  parser_context::ParserContext,
};

pub fn compile_script<'a>(_: &'a str) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new_from_file("./tests/uulang/test01.uu").unwrap();
  let parser_context = ParserContext::new(&parser, "Script");
  let pattern = crate::Script!();
  let program = crate::matchers::program::ProgramPattern::new_blank_program(
    crate::matchers::program::MatchAction::Continue,
  );

  let result = pattern.borrow().exec(parser_context.clone());
  match result {
    Ok(result) => match result {
      MatcherSuccess::Token(token) => {
        let token = token.borrow();
        println!("{:?}", token);
      }
      MatcherSuccess::ExtractChildren(token) => {
        let token = token.borrow();
        println!("{:?}", token);
      }
      MatcherSuccess::Skip(_) => {
        return Err("Internal Error(Skip): Invalid syntax".to_string());
      }
      MatcherSuccess::Break(_) => {
        return Err("Internal Error(Break): Invalid syntax".to_string());
      }
      MatcherSuccess::Continue(_) => {
        return Err("Internal Error(Continue): Invalid syntax".to_string());
      }
      MatcherSuccess::None => {
        return Err("Internal Error(None): Invalid syntax".to_string());
      }
      MatcherSuccess::Stop => {
        return Err("Internal Error(Stop): Invalid syntax".to_string());
      }
    },
    Err(error) => match error {
      crate::matcher::MatcherFailure::Fail => {
        return Err("Invalid syntax".to_string());
      }
      crate::matcher::MatcherFailure::Error(error) => {
        return Err(format!("Error: {}", error));
      }
    },
  }

  Ok(program)
}
