use std::cell::RefCell;

use crate::{
  matcher::{MatcherRef, MatcherSuccess},
  matchers::{
    program::{MatchAction, ProgramPattern},
    register::RegisterPattern,
  },
  parser::{Parser, ParserRef},
  parser_context::{ParserContext, ParserContextRef},
  token::TokenRef,
  Visit,
};

fn construct_matcher_from_pattern_token<'a>(
  token: TokenRef,
  name: &str,
  capture: bool,
) -> Result<MatcherRef<'a>, String> {
  let token = token.borrow();
  let token_name = token.get_name();
  let mut matcher: MatcherRef;

  if token_name == "EqualsMatcher" {
    let value = token.get_children()[0].borrow().value();
    if value == "" {
      return Err("Value can not be empty for an `Equals` pattern definition".to_string());
    }

    matcher = crate::Equals!(value);
  } else {
    return Err("Unkown pattern type".to_string());
  }

  if name != "" {
    matcher.borrow_mut().set_name(name);
  }

  if !capture {
    matcher = crate::Discard!(matcher);
  }

  Ok(matcher)
}

fn build_matcher_from_tokens<'a, 'b>(
  root_token: TokenRef,
  parser_context: ParserContextRef<'a>,
) -> Result<MatcherRef<'a>, String> {
  if root_token.borrow().get_name() != "Script" {
    return Err("Expected provided token to be a `Script` token".to_string());
  }

  let mut root_matcher = ProgramPattern::new_blank_program(MatchAction::Continue);
  let mut register_matchers = RefCell::new(RegisterPattern::new_blank());
  let mut current_matcher = &root_matcher;

  root_matcher
    .borrow_mut()
    .add_pattern(register_matchers.borrow().clone());

  println!("{:?}", root_token);

  let result = Visit!(root_token, program,
    "Script" => |_| {
      current_matcher = &root_matcher;
      Ok(())
    },
    "AssignmentExpression" => |token| {
      println!("Registering matcher!");

      // The name of the stored matcher is in the first child
      let token = token.borrow();
      let identifier = token.get_children()[0].borrow();
      let matcher_name = identifier.get_name();

      match parser_context.borrow().get_registered_matcher(matcher_name) {
        Some(matcher) => {
          let register_matchers = register_matchers.borrow();
          register_matchers.borrow_mut().add_pattern(matcher);
          Ok(())
        },
        None => Err(format!("Unable to fetch matcher by name `{}`", matcher_name)),
      }
    },
    "PatternScope" => |_| {
      Ok(())
    }
  );

  match result {
    Ok(_) => Ok(root_matcher),
    Err(error) => panic!("{}", error),
  }
}

pub fn compile_script<'a>(parser: ParserRef) -> Result<MatcherRef<'a>, String> {
  let parser_context = ParserContext::new(&parser, "Script");
  let pattern = crate::Script!();

  let result = pattern.borrow().exec(parser_context.clone());
  match result {
    Ok(result) => match result {
      MatcherSuccess::Token(token) => {
        build_matcher_from_tokens(token.clone(), parser_context.clone())
      }
      MatcherSuccess::ExtractChildren(token) => {
        build_matcher_from_tokens(token.clone(), parser_context.clone())
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
}

pub fn compile_script_from_str<'a>(source: &'a str) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new(source);
  compile_script(parser)
}

pub fn compile_script_from_file<'a>(file_name: &'a str) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new_from_file(file_name).unwrap();
  compile_script(parser)
}

// #[cfg(test)]
// mod tests {
//   use super::compile_script_from_file;

//   #[test]
//   fn it_works() {
//     if let Ok(result) = compile_script_from_file("./src/script/v1/tests/script/test01.axo") {
//     } else {
//       unreachable!("Test failed!");
//     };
//   }
// }
