use crate::{
  matcher::{MatcherRef, MatcherSuccess},
  matchers::program::{MatchAction, ProgramPattern},
  parser::Parser,
  parser_context::ParserContext,
};

pub fn compile_script_from_file<'a>(file_name: &'a str) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new_from_file(file_name).unwrap();
  let parser_context = ParserContext::new(&parser, "Script");
  let pattern = crate::Script!();
  let program = ProgramPattern::new_blank_program(MatchAction::Continue);

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

#[cfg(test)]
mod tests {
  use super::compile_script_from_file;

  #[test]
  fn it_works() {
    if let Ok(result) = compile_script_from_file("./src/script/v1/tests/script/test01.axo") {
    } else {
      unreachable!("Test failed!");
    };
  }
}
