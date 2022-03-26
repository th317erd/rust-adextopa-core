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
  Loop, Not, Optional, Visit,
};

use super::matchers::repeat_specifier::get_repeat_specifier_range;

lazy_static::lazy_static! {
  static ref PATTERN_MATCHER: regex::Regex = regex::Regex::new(r"^(CustomMatcher|EqualsMatcher|RegexMatcher|SequenceMatcher|ProgramMatcher|SwitchMatcher)$").expect("Could not compile needed Regex for `script::Parser`");
}

lazy_static::lazy_static! {
  static ref REPEAT_SPECIFIER: regex::Regex = regex::Regex::new(r"^(RepeatZeroOrMore|RepeatOneOrMore|RepeatRange)$").expect("Could not compile needed Regex for `script::Parser`");
}

fn construct_matcher_from_inner_definition<'a>(
  parser_context: ParserContextRef<'a>,
  matcher_token: TokenRef,
) -> Result<MatcherRef<'a>, String> {
  let matcher_token = matcher_token.borrow();
  let matcher_token_name = matcher_token.get_name();

  if matcher_token_name == "EqualsMatcher" {
    let value = matcher_token.get_children()[0].borrow().value();
    if value == "" {
      return Err("Value can not be empty for an `Equals` pattern definition".to_string());
    }

    Ok(crate::Equals!(value))
  } else {
    Err("Unkown pattern type".to_string())
  }
}

fn construct_matcher_from_pattern_definition<'a>(
  parser_context: ParserContextRef<'a>,
  token: TokenRef,
  name: &str,
  captured: bool,
) -> Result<MatcherRef<'a>, String> {
  let token = token.borrow();
  let token_name = token.get_name();
  let mut matcher: MatcherRef;

  if token_name != "PatternDefinition" {
    return Err(format!(
      "Expected a `PatternDefinition` token, but received a `{}` token instead",
      token_name
    ));
  }

  let has_outter_optional = token.has_child("OuterOptionalModifier");
  let has_outter_not = token.has_child("OuterNotModifier");
  let has_inner_optional = token.has_child("InnerOptionalModifier");
  let has_inner_not = token.has_child("InnerNotModifier");

  let matcher_token_result = token.find_child_fuzzy(&PATTERN_MATCHER);
  if matcher_token_result.is_none() {
    return Err("Unexpected pattern token encountered".to_string());
  }

  let matcher_token = matcher_token_result.unwrap();
  let mut matcher =
    construct_matcher_from_inner_definition(parser_context.clone(), matcher_token.clone())?;

  if name != "" {
    matcher.borrow_mut().set_name(name);
  }

  if has_inner_optional {
    matcher = Optional!(matcher);
  } else if has_inner_not {
    matcher = Not!(matcher);
  }

  if let Some(repeat_range) = token.find_child_fuzzy(&REPEAT_SPECIFIER) {
    let range = match get_repeat_specifier_range(repeat_range.clone()) {
      Ok(result) => result,
      Err(error) => return Err(error),
    };

    matcher = Loop!(range; matcher);
  }

  if has_outter_optional {
    matcher = Optional!(matcher);
  } else if has_outter_not {
    matcher = Not!(matcher);
  }

  if !captured {
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

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess,
    parser::Parser,
    parser_context::{ParserContext, ParserContextRef},
    source_range::SourceRange,
    ScriptPattern, ScriptPatternDefinition, ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  use super::{compile_script_from_file, construct_matcher_from_pattern_definition};

  // #[test]
  // fn it_works() {
  //   if let Ok(result) = compile_script_from_file("./src/script/v1/tests/script/test01.axo") {
  //   } else {
  //     unreachable!("Test failed!");
  //   };
  // }

  fn register_matchers(parser_context: &ParserContextRef) {
    (*parser_context)
      .borrow()
      .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);
  }

  #[test]
  fn it_can_construct_an_equals_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"<='test'>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternDefinition!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      println!("{:?}", token.borrow());
      let recreated_matcher =
        construct_matcher_from_pattern_definition(parser_context.clone(), token.clone(), "", false);

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(recreated_matcher.get_name(), "Discard");
      assert_eq!(recreated_matcher.get_children().unwrap().len(), 1);

      let children = recreated_matcher.get_children().unwrap();
      let child = children[0].borrow();
      assert_eq!(
        child.to_string(),
        "EqualsPattern { pattern: \"test\", name: \"Equals\", custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
