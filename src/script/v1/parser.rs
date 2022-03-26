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
  Loop, Not, Optional, ScriptProgramMatcher, ScriptSwitchMatcher, Visit,
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
  } else if matcher_token_name == "RegexMatcher" {
    let mut value = matcher_token.get_children()[0].borrow().value();
    if value == "" {
      return Err("Value can not be empty for a `Matches` pattern definition".to_string());
    }

    if let Some(flags) = matcher_token.find_child("Flags") {
      value = format!("(?{}){}", flags.borrow().value(), &value);
    }

    Ok(crate::Matches!(&value))
  } else if matcher_token_name == "SequenceMatcher" {
    let start_pattern = matcher_token.get_children()[0].borrow();
    let end_pattern = matcher_token.get_children()[1].borrow();
    let escape_pattern = matcher_token.get_children()[2].borrow();

    if start_pattern.value().len() == 0 {
      panic!("Sequence `start` pattern of \"\" makes no sense");
    }

    if end_pattern.value().len() == 0 {
      panic!("Sequence `end` pattern of \"\" makes no sense");
    }

    Ok(crate::Sequence!(
      start_pattern.value(),
      end_pattern.value(),
      escape_pattern.value()
    ))
  } else if matcher_token_name == "CustomMatcher" {
    let identifier = matcher_token.get_children()[0].borrow().value();
    if identifier == "" {
      return Err(
        "Identifier can not be empty for a `CustomMatcher` pattern definition".to_string(),
      );
    }

    Ok(crate::Ref!(identifier))
  } else if matcher_token_name == "SwitchMatcher" {
    let children = matcher_token.get_children();

    // If this switch has no children...
    // then just return a Null matcher instead
    if children.len() == 0 {
      return Ok(crate::Null!());
    }

    let switch_matcher = crate::Switch!();
    let mut _switch_matcher = switch_matcher.borrow_mut();
    for child in children {
      let matcher = construct_matcher_from_pattern(parser_context.clone(), child.clone())?;
      _switch_matcher.add_pattern(matcher);
    }

    drop(_switch_matcher);

    Ok(switch_matcher)
  } else if matcher_token_name == "ProgramMatcher" {
    let children = matcher_token.get_children();

    // If this switch has no children...
    // then just return a Null matcher instead
    if children.len() == 0 {
      return Ok(crate::Null!());
    }

    let program_matcher = crate::Program!();
    let mut _program_matcher = program_matcher.borrow_mut();
    for child in children {
      let matcher = construct_matcher_from_pattern(parser_context.clone(), child.clone())?;
      _program_matcher.add_pattern(matcher);
    }

    drop(_program_matcher);

    Ok(program_matcher)
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

fn construct_matcher_from_pattern<'a>(
  parser_context: ParserContextRef<'a>,
  token: TokenRef,
) -> Result<MatcherRef<'a>, String> {
  let _token = token.borrow();
  let token_name = _token.get_name();

  if token_name == "PatternDefinitionCaptured" {
    if let Some(name_token) = _token.find_child("MatcherName") {
      let name_token = name_token.borrow();
      let name_child = name_token.get_children()[0].borrow();
      let name = name_child.get_name();

      construct_matcher_from_pattern_definition(
        parser_context,
        _token.get_children()[1].clone(),
        name,
        true,
      )
    } else {
      construct_matcher_from_pattern_definition(
        parser_context,
        _token.get_children()[0].clone(),
        "",
        true,
      )
    }
  } else if token_name == "PatternDefinition" {
    construct_matcher_from_pattern_definition(parser_context, token.clone(), "", false)
  } else {
    Err(format!(
      "Expected a `PatternDefinitionCaptured`, or `PatternDefinition` token, but received a `{}` token instead",
      token_name
    ))
  }
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

  (*parser_context)
    .borrow()
    .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);

  let pattern = crate::Script!();

  let result = ParserContext::tokenize(parser_context.clone(), pattern);
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
    script::current::parser::construct_matcher_from_pattern,
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
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

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

  #[test]
  fn it_can_construct_an_equals_matcher_from_a_pattern_token2() {
    let parser = Parser::new(r"(<='test'>{2,3})");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(
        recreated_matcher.to_string(),
        "LoopPattern { patterns: [RefCell { value: EqualsPattern { pattern: \"test\", name: \"Equals\", custom_name: false } }], name: \"Loop\", iterate_range: Some(2..3), on_first_match: Continue, custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_an_equals_matcher_from_a_pattern_token3() {
    let parser = Parser::new(r"(?'Test'<='test'>+)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(
        recreated_matcher.to_string(),
        "LoopPattern { patterns: [RefCell { value: EqualsPattern { pattern: \"test\", name: \"Name\", custom_name: true } }], name: \"Loop\", iterate_range: Some(1..18446744073709551615), on_first_match: Continue, custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_a_regex_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"</test/i>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternDefinition!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
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
        "MatchesPattern { regex: (?i)test, name: \"Matches\", custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_a_sequence_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"<%'[',']','\\'>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternDefinition!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
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
        "SequencePattern { start: \"[\", end: \"]\", escape: \"\\\\\", name: \"Sequence\", custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_a_custom_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"<Test>");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPatternDefinition!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher =
        construct_matcher_from_pattern_definition(parser_context.clone(), token.clone(), "", false);

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(recreated_matcher.get_name(), "Discard");
      assert_eq!(recreated_matcher.get_children().unwrap().len(), 1);

      let children = recreated_matcher.get_children().unwrap();
      let child = children[0].borrow();
      assert_eq!(child.to_string(), "RefPattern { target_name: \"Test\" }");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_a_switch_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"(<[<='test'>|(?'Derp'</wow/i>)]>)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(
        recreated_matcher.to_string(),
        "SwitchPattern { patterns: [RefCell { value: DiscardPattern { matcher: RefCell { value: EqualsPattern { pattern: \"test\", name: \"Equals\", custom_name: false } } } }, RefCell { value: MatchesPattern { regex: (?i)wow, name: \"Name\", custom_name: true } }], name: \"Switch\", iterate_range: None, on_first_match: Stop, custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_construct_a_program_matcher_from_a_pattern_token1() {
    let parser = Parser::new(r"(<{<='test'>(?'Derp'</wow/i>)}>)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.borrow();

      assert_eq!(
        recreated_matcher.to_string(),
        "ProgramPattern { patterns: [RefCell { value: DiscardPattern { matcher: RefCell { value: EqualsPattern { pattern: \"test\", name: \"Equals\", custom_name: false } } } }, RefCell { value: MatchesPattern { regex: (?i)wow, name: \"Name\", custom_name: true } }], name: \"Program\", iterate_range: None, on_first_match: Continue, custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
