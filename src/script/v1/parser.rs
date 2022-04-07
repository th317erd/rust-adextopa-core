use std::{collections::HashMap, path::Path};

use crate::{
  matcher::MatcherRef,
  matchers::{
    program::{MatchAction, ProgramPattern},
    register::RegisterPattern,
  },
  parser::{Parser, ParserRef},
  parser_context::{ParserContext, ParserContextRef},
  token::TokenRef,
  Loop, Not, Optional, ScriptProgramMatcher, ScriptSwitchMatcher, SetScope, Visit,
};

use super::matchers::repeat_specifier::get_repeat_specifier_range;

lazy_static::lazy_static! {
  static ref PATTERN_MATCHER: regex::Regex = regex::Regex::new(r"^(CustomMatcher|EqualsMatcher|RegexMatcher|SequenceMatcher|ProgramMatcher|SwitchMatcher)$").expect("Could not compile needed Regex for `script::Parser`");
}

lazy_static::lazy_static! {
  static ref REPEAT_SPECIFIER: regex::Regex = regex::Regex::new(r"^(RepeatZeroOrMore|RepeatOneOrMore|RepeatRange)$").expect("Could not compile needed Regex for `script::Parser`");
}

fn construct_matcher_from_inner_definition(
  parser_context: ParserContextRef,
  matcher_token: TokenRef,
) -> Result<MatcherRef, String> {
  let matcher_token = matcher_token.borrow();
  let matcher_token_name = matcher_token.get_name();

  if matcher_token_name == "EqualsMatcher" {
    let value: String = matcher_token.get_children()[0].borrow().get_value().clone();

    if value == "" {
      return Err("Value can not be empty for an `Equals` pattern definition".to_string());
    }

    Ok(crate::Equals!(value))
  } else if matcher_token_name == "RegexMatcher" {
    let mut value: String = matcher_token.get_value().clone();

    if value == "" {
      return Err("Value can not be empty for a `Matches` pattern definition".to_string());
    }

    if let Some(flags) = matcher_token.find_child("Flags") {
      value = format!("(?{}){}", flags.borrow().get_captured_value(), value);
    }

    Ok(crate::Matches!(&value))
  } else if matcher_token_name == "SequenceMatcher" {
    let start_pattern = matcher_token.get_children()[0].borrow();
    let end_pattern = matcher_token.get_children()[1].borrow();
    let escape_pattern = matcher_token.get_children()[2].borrow();

    if start_pattern.get_value().len() == 0 {
      panic!("Sequence `start` pattern of \"\" makes no sense");
    }

    if end_pattern.get_value().len() == 0 {
      panic!("Sequence `end` pattern of \"\" makes no sense");
    }

    Ok(crate::Sequence!(
      start_pattern.get_value().clone(),
      end_pattern.get_value().clone(),
      escape_pattern.get_value().clone()
    ))
  } else if matcher_token_name == "CustomMatcher" {
    let identifier = matcher_token.get_children()[0].borrow().get_value().clone();

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
      _switch_matcher.add_pattern(matcher.0);
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
      _program_matcher.add_pattern(matcher.0);
    }

    drop(_program_matcher);

    Ok(program_matcher)
  } else {
    Err("Unkown pattern type".to_string())
  }
}

fn construct_matcher_from_pattern_definition(
  parser_context: ParserContextRef,
  token: TokenRef,
  name: &str,
  captured: bool,
) -> Result<(MatcherRef, MatcherRef), String> {
  let token = token.borrow();
  let token_name = token.get_name();

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
  let inner_matcher = matcher.clone();

  if name != "" {
    matcher.borrow_mut().set_name(name);
  }

  // Handle attributes with a "Map" matcher
  if let Some(attributes_token) = token.find_child("Attributes") {
    let attributes_token = attributes_token.borrow();
    let attributes_token_children = attributes_token.get_children();

    if attributes_token_children.len() > 0 {
      // First, collect attributes from token into a map
      let mut attributes = HashMap::<String, String>::new();
      for child in attributes_token_children {
        let child = child.borrow();
        let child_children = child.get_children();
        let key = child_children[0].borrow().get_captured_value().clone();
        let value = child_children[1].borrow().get_captured_value().clone();

        attributes.insert(key, value);
      }

      // Next, move the attributes hashmap into the
      // "Map" matcher, to apply the attributes
      // to a generated token
      matcher = crate::Map!(matcher.clone(), move |token| {
        for attribute in &attributes {
          let key = attribute.0;
          if token.borrow().has_attribute(key) {
            continue;
          }

          let value = attribute.1;
          token.borrow_mut().set_attribute(key, value);
        }

        None
      });
    }
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

  Ok((matcher, inner_matcher))
}

fn construct_matcher_from_pattern(
  parser_context: ParserContextRef,
  token: TokenRef,
) -> Result<(MatcherRef, MatcherRef), String> {
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

fn build_matcher_from_tokens(
  root_token: TokenRef,
  parser_context: ParserContextRef,
  name: String,
  from_file: Option<&str>,
) -> Result<MatcherRef, String> {
  if root_token.borrow().get_name() != "Script" {
    return Err("Expected provided token to be a `Script` token".to_string());
  }

  let root_matcher = ProgramPattern::new_program_with_name(Vec::new(), name, MatchAction::Continue);
  let register_matchers = RegisterPattern::new_blank();

  root_matcher
    .borrow_mut()
    .add_pattern(register_matchers.clone());

  let result = Visit!(root_token, program,
    "AssignmentExpression" => |token| {
      // The name of the stored matcher is in the first child
      let _token = token.borrow();
      let token_children = _token.get_children();
      let identifier = token_children[0].borrow();
      let matcher_name = identifier.get_value().clone();
      let value = &token_children[1];
      let _value = value.borrow();
      let value_name = _value.get_name();

      if value_name == "Identifier" {
        // Identifier is assigned to identifier...
        // so this is a reference
        register_matchers.borrow_mut().add_pattern(crate::Ref!(matcher_name; _token.get_value().clone()));
      } else if value_name == "PatternDefinition" {
        // This is a pattern definition, so turn it into
        // a matcher, and store it as a reference
        match construct_matcher_from_pattern_definition(parser_context.clone(), value.clone(), &matcher_name, true) {
          Ok(defined_matchers) => {
            register_matchers.borrow_mut().add_pattern(crate::Ref!(matcher_name; defined_matchers.0.clone()));
          },
          Err(error) => {
            // TODO: Better error handling
            eprintln!("Error attempting to create matcher from parsed pattern token: {}", error);
          }
        }
      }

      Ok(())
    },
    "ImportStatement" => |token| {
      if from_file.is_none() {
        // TODO: Better error handling
        let error = "Error found import statement, but it is forbidden to use imports when not parsing from a file";

        eprintln!("{}", error);

        return Err(error.to_string());
      }

      let from_file = from_file.as_ref().unwrap();
      let token = token.borrow();
      let import_identifiers = token.find_child("ImportIdentifiers").unwrap();
      let import_identifiers = import_identifiers.borrow();
      let import_identifiers = import_identifiers.get_children();
      let path = token.find_child("Path").unwrap().borrow().get_value().clone();

      let full_path = Path::new(from_file).parent().unwrap().join(path).canonicalize().unwrap();
      let file_name = full_path.to_str().unwrap();

      let import_result = compile_script_from_file_internal(file_name)?;
      let import_parser_context = import_result.0.borrow();
      let import_root_matcher = &import_result.1;
      let _parser_context = parser_context.borrow();
      let mut register_matchers = register_matchers.borrow_mut();

      for import_identifier in import_identifiers {
        let import_identifier = import_identifier.borrow();
        let identifier = import_identifier.find_child("ImportName").unwrap().borrow().get_value().to_string();
        let as_name = import_identifier.find_child("ImportAsName");
        let import_name = match as_name {
          Some(child) => child.borrow().get_value().to_string(),
          None => identifier.clone(),
        };

        if identifier == "_" {
          println!("Registering root matcher from import: {}", import_name);
          register_matchers.add_pattern(crate::Ref!(import_name; SetScope!(import_parser_context.scope.clone(), import_root_matcher.clone())));
        } else {
          let reference_matcher = import_parser_context.get_registered_matcher(&identifier);
          if reference_matcher.is_none() {
            return Err(format!("Failed to import `{}` from '{}': Not found", &identifier, file_name))
          }

          println!("Registering matcher from import: {}", import_name);
          register_matchers.add_pattern(crate::Ref!(import_name; SetScope!(import_parser_context.scope.clone(), reference_matcher.unwrap().clone())));
        }
      }

      Ok(())
    },
    "PatternScope" => |token| {
      let token = token.borrow();
      let children = token.get_children();

      for child in children {
        let _child = child.borrow();
        let child_name = _child.get_name();

        if child_name == "PatternDefinitionCaptured" || child_name == "PatternDefinition" {
          match construct_matcher_from_pattern(parser_context.clone(), child.clone()) {
            Ok(defined_matchers) => {
              root_matcher.borrow_mut().add_pattern(defined_matchers.0);
            },
            Err(error) => {
              // TODO: Better error handling
              eprintln!("Error attempting to create matcher from parsed pattern token: {}", error);
            }
          }
        }
      }

      Ok(())
    }
  );

  match result {
    Ok(_) => Ok(root_matcher),
    Err(error) => panic!("{}", error),
  }
}

pub fn compile_script(
  parser: ParserRef,
  name: String,
  from_file: Option<&str>,
) -> Result<(ParserContextRef, MatcherRef), String> {
  let parser_context = ParserContext::new(&parser, &name);

  (*parser_context)
    .borrow()
    .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);

  let pattern = crate::Script!();

  let result = ParserContext::tokenize(parser_context.clone(), pattern);
  match result {
    Ok(token) => {
      match build_matcher_from_tokens(token.clone(), parser_context.clone(), name, from_file) {
        Ok(ref matcher) => {
          parser_context
            .borrow()
            .capture_matcher_references(matcher.clone());

          Ok((parser_context.clone(), matcher.clone()))
        }
        Err(error) => Err(error),
      }
    }
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

pub fn compile_script_from_str(source: &str, name: String) -> Result<MatcherRef, String> {
  let parser = Parser::new(source);
  match compile_script(parser, name, None) {
    Ok(result) => Ok(result.1),
    Err(error) => Err(error),
  }
}

fn compile_script_from_file_internal(
  file_name: &str,
) -> Result<(ParserContextRef, MatcherRef), String> {
  let full_path = Path::new(file_name).canonicalize().unwrap();
  let full_file_name = full_path.to_str().unwrap();

  let parser = Parser::new_from_file(full_file_name).unwrap();
  compile_script(parser, file_name.to_string(), Some(full_file_name))
}

pub fn compile_script_from_file(file_name: &str) -> Result<MatcherRef, String> {
  let full_path = Path::new(file_name).canonicalize().unwrap();
  let full_file_name = full_path.to_str().unwrap();

  let parser = Parser::new_from_file(full_file_name).unwrap();
  match compile_script(parser, file_name.to_string(), Some(full_file_name)) {
    Ok(result) => Ok(result.1),
    Err(error) => Err(error),
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    parser::Parser,
    parser_context::{ParserContext, ParserContextRef},
    script::current::parser::construct_matcher_from_pattern,
    source_range::SourceRange,
    ScriptPattern, ScriptPatternDefinition, ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  use super::{compile_script_from_file, construct_matcher_from_pattern_definition};

  #[test]
  fn it_compiles_a_script_and_returns_a_matcher() {
    if let Ok(compiled_matcher) =
      compile_script_from_file("./src/script/v1/tests/script/test_word.axo")
    {
      let parser = Parser::new("test");
      let parser_context = ParserContext::new(&parser, "Test");

      // println!("MATCHER: {:?}", compiled_matcher);
      // let compiled_matcher = Debug!(compiled_matcher);

      let result = ParserContext::tokenize(parser_context, compiled_matcher.clone());

      if let Ok(token) = result {
        let token = token.borrow();

        assert_eq!(
          token.get_name(),
          "./src/script/v1/tests/script/test_word.axo"
        );
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 4));
        assert_eq!(*token.get_matched_range(), SourceRange::new(0, 4));
        assert_eq!(token.get_value(), "test");
        assert_eq!(token.get_matched_value(), "test");
        assert_eq!(token.get_children().len(), 1);

        let first = token.get_children()[0].borrow();
        assert_eq!(first.get_name(), "Word");
        assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
        assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
        assert_eq!(first.get_value(), "test");
        assert_eq!(first.get_matched_value(), "test");

        assert_eq!(first.get_attribute("hello"), Some(&"world".to_string()));
      } else {
        unreachable!("Test failed!");
      };
    } else {
      unreachable!("Test failed!");
    };
  }

  // #[test]
  // fn it_compiles_a_script_with_an_import_and_returns_a_matcher() {
  //   if let Ok(compiled_matcher) =
  //     compile_script_from_file("./src/script/v1/tests/script/test_import.axo")
  //   {
  //     let parser = Parser::new("hello world");
  //     let parser_context = ParserContext::new(&parser, "Test");

  //     // println!("MATCHER: {:?}", compiled_matcher);
  //     // let compiled_matcher = Debug!(compiled_matcher);

  //     let result = ParserContext::tokenize(parser_context, compiled_matcher.clone());

  //     if let Ok(token) = result {
  //       let token = token.borrow();

  //       assert_eq!(
  //         token.get_name(),
  //         "./src/script/v1/tests/script/test_import.axo"
  //       );
  //       assert_eq!(*token.get_captured_range(), SourceRange::new(0, 5));
  //       assert_eq!(*token.get_matched_range(), SourceRange::new(0, 5));
  //       assert_eq!(token.get_value(), "hello");
  //       assert_eq!(token.get_matched_value(), "hello");
  //       assert_eq!(token.get_children().len(), 1);

  //       let first = token.get_children()[0].borrow();
  //       assert_eq!(first.get_name(), "Word");
  //       assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
  //       assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
  //       assert_eq!(first.get_value(), "test");
  //       assert_eq!(first.get_matched_value(), "test");

  //       assert_eq!(first.get_attribute("hello"), Some(&"world".to_string()));
  //     } else {
  //       println!("ERROR: {:?}", result);
  //       unreachable!("Test failed!");
  //     };
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

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher =
        construct_matcher_from_pattern_definition(parser_context.clone(), token.clone(), "", false);

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher =
        construct_matcher_from_pattern_definition(parser_context.clone(), token.clone(), "", false);

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher =
        construct_matcher_from_pattern_definition(parser_context.clone(), token.clone(), "", false);

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

      assert_eq!(recreated_matcher.get_name(), "Discard");
      assert_eq!(recreated_matcher.get_children().unwrap().len(), 1);

      let children = recreated_matcher.get_children().unwrap();
      let child = children[0].borrow();
      assert_eq!(
        child.to_string(),
        "RefPattern { name: \"Ref\", target: \"Test\", custom_name: false }"
      );
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

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

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

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap();
      let recreated_matcher = recreated_matcher.0.borrow();

      assert_eq!(
        recreated_matcher.to_string(),
        "ProgramPattern { patterns: [RefCell { value: DiscardPattern { matcher: RefCell { value: EqualsPattern { pattern: \"test\", name: \"Equals\", custom_name: false } } } }, RefCell { value: MatchesPattern { regex: (?i)wow, name: \"Name\", custom_name: true } }], name: \"Program\", iterate_range: None, on_first_match: Continue, custom_name: false }"
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_collect_attributes_from_a_token() {
    let parser = Parser::new(r"(</test/i test='1' hello='derp'>)");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptPattern!();

    register_matchers(&parser_context);

    let result = ParserContext::tokenize(parser_context.clone(), matcher);

    if let Ok(token) = result {
      let recreated_matcher = construct_matcher_from_pattern(parser_context.clone(), token.clone());

      assert_eq!(recreated_matcher.is_ok(), true);
      let recreated_matcher = recreated_matcher.unwrap().0;

      assert_eq!(
        recreated_matcher.borrow().to_string(),
        "MapPattern { matcher: RefCell { value: MatchesPattern { regex: (?i)test, name: \"Matches\", custom_name: false } } }"
      );

      // Now see if we can use this matcher
      let parser2 = Parser::new(r"test");
      let parser_context2 = ParserContext::new(&parser2, "Test");

      register_matchers(&parser_context2);

      let result2 = ParserContext::tokenize(parser_context2.clone(), recreated_matcher);

      if let Ok(token) = result2 {
        let token = token.borrow();
        assert_eq!(token.get_name(), "Matches");
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 4));
        assert_eq!(*token.get_matched_range(), SourceRange::new(0, 4));
        assert_eq!(token.get_value(), "test");
        assert_eq!(token.get_matched_value(), "test");

        let attributes = token.get_attributes();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes.get("test"), Some(&"1".to_string()));
        assert_eq!(attributes.get("hello"), Some(&"derp".to_string()));
      } else {
        unreachable!("Test failed!");
      };
    } else {
      unreachable!("Test failed!");
    };
  }
}
