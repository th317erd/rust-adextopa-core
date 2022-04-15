use std::{cell::RefCell, collections::HashMap, path::Path};

use crate::{
  matcher::MatcherRef,
  matchers::program::{MatchAction, ProgramPattern},
  parse_error::ParseError,
  parser::{Parser, ParserRef},
  parser_context::{ParserContext, ParserContextRef},
  scope::VariableType,
  scope_context::{ScopeContext, ScopeContextRef},
  source_range::SourceRange,
  token::TokenRef,
  Loop, Map, Not, Optional, ProxyChildren, Ref, ScriptProgramMatcher, ScriptSwitchMatcher,
  SetScope, TokenResult, Visit,
};

use super::matchers::repeat_specifier::get_repeat_specifier_range;

lazy_static::lazy_static! {
  static ref PATTERN_MATCHER: regex::Regex = regex::Regex::new(r"^(CustomMatcher|EqualsMatcher|RegexMatcher|SequenceMatcher|ProgramMatcher|SwitchMatcher)$").expect("Could not compile needed Regex for `script::Parser`");
}

lazy_static::lazy_static! {
  static ref REPEAT_SPECIFIER: regex::Regex = regex::Regex::new(r"^(RepeatZeroOrMore|RepeatOneOrMore|RepeatRange)$").expect("Could not compile needed Regex for `script::Parser`");
}

pub const FLAG_LOG_STDERR: u32 = 0x01;

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
      matcher = crate::Map!(matcher.clone(), move |token, _, __| {
        for attribute in &attributes {
          let key = attribute.0;
          if token.borrow().has_attribute(key) {
            continue;
          }

          let value = attribute.1;
          token.borrow_mut().set_attribute(key, value);
        }

        TokenResult!(token.clone())
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
  flags: u32,
) -> Result<(MatcherRef, ScopeContextRef), Vec<ParseError>> {
  if root_token.borrow().get_name() != "Script" {
    return Err(vec![ParseError::new(
      "Expected provided token to be a `Script` token",
    )]);
  }

  let root_matcher = ProgramPattern::new_program_with_name(Vec::new(), name, MatchAction::Continue);
  let scoped_matcher = ProgramPattern::new_program(Vec::new(), MatchAction::Continue);
  let scope_context = ScopeContext::new();
  let parse_errors = RefCell::new(Vec::<ParseError>::new());

  root_matcher.borrow_mut().add_pattern(SetScope!(
    scope_context.clone(),
    ProxyChildren!(scoped_matcher.clone())
  ));

  let result = Visit!(root_token, program,
    "Error" => |token| {
      let _token = token.borrow();

      let error_message = _token.get_attribute("__message").unwrap();
      parse_errors.borrow_mut().push(ParseError::new_with_range(&error_message, _token.get_matched_range().clone()));

      Ok(())
    },
    "AdextopaScope" => |token| {
      let _token = token.borrow();

      for (name, value) in _token.get_attributes().iter() {
        if name == "name" {
          root_matcher.borrow_mut().set_name(value);
        }
      }

      Ok(())
    },
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
        scope_context.borrow_mut().set(&matcher_name, VariableType::Matcher(Ref!(_token.get_value().clone())));
      } else if value_name == "PatternDefinition" {
        // This is a pattern definition, so turn it into
        // a matcher, and store it as a reference
        match construct_matcher_from_pattern_definition(parser_context.clone(), value.clone(), &matcher_name, true) {
          Ok(defined_matchers) => {
            scope_context.borrow_mut().set(&matcher_name, VariableType::Matcher(defined_matchers.0.clone()));
          },
          Err(error) => {
            return Err(ParseError::new(&format!("Error attempting to create matcher from parsed pattern token: {}", error)));
          }
        }
      }

      Ok(())
    },
    "ImportStatement" => |token| {
      if from_file.is_none() {
        return Err(ParseError::new_with_range("Found import statement, but it is forbidden to use imports when not parsing from a file", token.borrow().get_matched_range().clone()));
      }

      let from_file = from_file.as_ref().unwrap();
      let _token = token.borrow();
      let import_identifiers = _token.find_child("ImportIdentifiers").unwrap();
      let import_identifiers = import_identifiers.borrow();
      let import_identifiers = import_identifiers.get_children();
      let path = _token.find_child("Path").unwrap().borrow().get_value().clone();

      let full_path = Path::new(from_file).parent().unwrap().join(path).canonicalize().unwrap();
      let file_name = full_path.to_str().unwrap();

      let import_result = match compile_script_from_file_internal(file_name, flags) {
        Ok(result) => result,
        Err(errors) => {
          let mut _parse_errors = parse_errors.borrow_mut();

          for error in errors {
            _parse_errors.push(error);
          }

          return Err(ParseError::new(""));
        },
      };

      let import_root_matcher = import_result.1;
      let import_scope_context = import_result.2;
      //let mut register_matchers = register_matcher.borrow_mut();

      for import_identifier in import_identifiers {
        let import_identifier = import_identifier.borrow();
        let identifier_token = import_identifier.find_child("ImportName").unwrap();
        let identifier = identifier_token.borrow().get_value().to_string();
        let as_name = import_identifier.find_child("ImportAsName");
        let import_name = match as_name {
          Some(child) => child.borrow().get_value().to_string(),
          None => identifier.clone(),
        };

        if identifier == "_" {
          if import_name == "_" {
            return Err(ParseError::new_with_range("Error root import '_' must be named using 'import {{ _ as Name }}'", identifier_token.borrow().get_matched_range().clone()));
          }

          import_root_matcher.borrow_mut().set_name(&import_name);

          // This should work all on its own, because this is the root pattern
          // and it implements its own scope when called
          scope_context.borrow_mut().set(&import_name, VariableType::Matcher(import_root_matcher.clone()));
        } else {
          let reference_matcher = import_scope_context.borrow().get(&identifier);

          match reference_matcher {
            Some(VariableType::Matcher(ref matcher)) => {
              let token_name = import_name.clone();

              let map_matcher = Map!(matcher.clone(), move |token, _, __| {
                let mut _token = token.borrow_mut();
                _token.set_name(&token_name);

                TokenResult!(token.clone())
              });

              // TODO: This will not work until we get the full scope used by the parser
              scope_context.borrow_mut().set(&import_name, VariableType::Matcher(SetScope!(import_scope_context.clone(), map_matcher)));
            },
            _ => return Err(ParseError::new_with_range(&format!("Failed to import `{}` from '{}': Not found", &identifier, file_name), identifier_token.borrow().get_matched_range().clone())),
          }
        }
      }

      Ok(())
    },
    "PatternScope" => |token| {
      let _token = token.borrow();
      let children = _token.get_children();

      for child in children {
        let _child = child.borrow();
        let child_name = _child.get_name();

        if child_name == "PatternDefinitionCaptured" || child_name == "PatternDefinition" {
          match construct_matcher_from_pattern(parser_context.clone(), child.clone()) {
            Ok(defined_matchers) => {
              scoped_matcher.borrow_mut().add_pattern(defined_matchers.0);
            },
            Err(error) => {
              return Err(ParseError::new(&format!("Error attempting to create matcher from parsed pattern token: {}", error)));
            }
          }
        }
      }

      Ok(())
    }
  );

  match result {
    Ok(_) => Ok((root_matcher, scope_context)),
    Err(error) => {
      if error.message != "" {
        let range = match &error.range {
          Some(range) => range.clone(),
          None => SourceRange::new(0, 0),
        };

        let error_message = parser_context
          .borrow()
          .get_error_as_string(&error.message, &range);

        parse_errors
          .borrow_mut()
          .push(ParseError::new_with_range(&error_message, range));
      }

      Err(parse_errors.into_inner())
    }
  }
}

pub fn log_errors_to_stdout(
  _: ParserContextRef,
  parsed_token: Option<TokenRef>,
  errors: Option<&Vec<ParseError>>,
) {
  match parsed_token {
    Some(token) => {
      let _token = token.borrow();
      let children = _token.get_children();

      for child in children {
        if child.borrow().get_name() == "Error" {
          eprintln!(
            "{}",
            child.borrow().get_attribute("__message").unwrap().as_str()
          );
        }
      }
    }
    None => {
      if errors.is_none() {
        return;
      }

      for error in errors.unwrap() {
        eprintln!("{}", error.message);
      }
    }
  }
}

pub fn compile_script(
  parser: ParserRef,
  name: String,
  from_file: Option<&str>,
  flags: u32,
) -> Result<(ParserContextRef, MatcherRef, ScopeContextRef), Vec<ParseError>> {
  let parser_context = ParserContext::new(&parser, &name);

  (*parser_context)
    .borrow()
    .register_matchers(vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);

  let pattern = crate::Script!();

  let result = ParserContext::tokenize(parser_context.clone(), pattern);

  match result {
    Ok(ref token) => {
      match build_matcher_from_tokens(
        token.clone(),
        parser_context.clone(),
        name,
        from_file,
        flags,
      ) {
        Ok(result) => {
          if flags & FLAG_LOG_STDERR > 0 {
            log_errors_to_stdout(parser_context.clone(), Some(token.clone()), None);
          }

          Ok((parser_context, result.0, result.1))
        }
        Err(errors) => {
          if flags & FLAG_LOG_STDERR > 0 {
            log_errors_to_stdout(parser_context.clone(), None, Some(&errors));
          }

          Err(errors)
        }
      }
    }
    Err(error) => match error {
      crate::matcher::MatcherFailure::Fail => {
        let errors = vec![ParseError::new("Failed to parse script with an unknown error. This is likely a bug. Please report this issue to the adextopa maintainers.")];

        if flags & FLAG_LOG_STDERR > 0 {
          log_errors_to_stdout(parser_context.clone(), None, Some(&errors));
        }

        Err(errors)
      }
      crate::matcher::MatcherFailure::Error(error) => {
        let errors = vec![error];

        if flags & FLAG_LOG_STDERR > 0 {
          log_errors_to_stdout(parser_context.clone(), None, Some(&errors));
        }

        Err(errors)
      }
    },
  }
}

pub fn compile_script_from_str(
  source: &str,
  name: String,
  flags: u32,
) -> Result<MatcherRef, Vec<ParseError>> {
  let parser = Parser::new(source);
  match compile_script(parser, name, None, flags) {
    Ok(result) => Ok(result.1),
    Err(errors) => Err(errors),
  }
}

fn compile_script_from_file_internal(
  file_name: &str,
  flags: u32,
) -> Result<(ParserContextRef, MatcherRef, ScopeContextRef), Vec<ParseError>> {
  let full_path = Path::new(file_name).canonicalize().unwrap();
  let full_file_name = full_path.to_str().unwrap();

  let parser = Parser::new_from_file(full_file_name).unwrap();
  compile_script(parser, file_name.to_string(), Some(full_file_name), flags)
}

pub fn compile_script_from_file(
  file_name: &str,
  flags: u32,
) -> Result<MatcherRef, Vec<ParseError>> {
  let full_path = Path::new(file_name).canonicalize().unwrap();
  let full_file_name = full_path.to_str().unwrap();

  let parser = Parser::new_from_file(full_file_name).unwrap();
  match compile_script(parser, file_name.to_string(), Some(full_file_name), flags) {
    Ok(result) => Ok(result.1),
    Err(errors) => Err(errors),
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
      compile_script_from_file("./src/script/v1/tests/script/test_word.axo", 0)
    {
      let parser = Parser::new("test");
      let parser_context = ParserContext::new(&parser, "Test");

      // println!("MATCHER: {:?}", compiled_matcher);
      // let compiled_matcher = Debug!(compiled_matcher);

      let result = ParserContext::tokenize(parser_context, compiled_matcher.clone());

      if let Ok(token) = result {
        let token = token.borrow();

        assert_eq!(token.get_name(), "Word");
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
        assert_eq!(first.get_children().len(), 0);

        assert_eq!(first.get_attribute("hello"), Some(&"world".to_string()));
      } else {
        unreachable!("Test failed!");
      };
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_compiles_a_script_with_an_import_and_returns_a_matcher() {
    if let Ok(compiled_matcher) =
      compile_script_from_file("./src/script/v1/tests/script/test_import.axo", 0)
    {
      let parser = Parser::new("hello world");
      let parser_context = ParserContext::new(&parser, "Test");

      // println!("MATCHER: {:?}", compiled_matcher);
      // let compiled_matcher = Debug!(3; compiled_matcher);

      let result = ParserContext::tokenize(parser_context, compiled_matcher.clone());

      if let Ok(token) = result {
        let token = token.borrow();

        assert_eq!(token.get_name(), "TestImport");
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
        assert_eq!(*token.get_matched_range(), SourceRange::new(0, 11));
        assert_eq!(token.get_value(), "hello world");
        assert_eq!(token.get_matched_value(), "hello world");
        assert_eq!(token.get_children().len(), 2);

        let first = token.get_children()[0].borrow();
        assert_eq!(first.get_name(), "Word");
        assert_eq!(*first.get_captured_range(), SourceRange::new(0, 5));
        assert_eq!(*first.get_matched_range(), SourceRange::new(0, 5));
        assert_eq!(first.get_value(), "hello");
        assert_eq!(first.get_matched_value(), "hello");
        assert_eq!(first.get_children().len(), 1);

        let first_child = first.get_children()[0].borrow();
        assert_eq!(first_child.get_name(), "Word");
        assert_eq!(*first_child.get_captured_range(), SourceRange::new(0, 5));
        assert_eq!(*first_child.get_matched_range(), SourceRange::new(0, 5));
        assert_eq!(first_child.get_value(), "hello");
        assert_eq!(first_child.get_matched_value(), "hello");
        assert_eq!(first_child.get_children().len(), 0);

        let second = token.get_children()[1].borrow();
        assert_eq!(second.get_name(), "Chunk");
        assert_eq!(*second.get_captured_range(), SourceRange::new(6, 11));
        assert_eq!(*second.get_matched_range(), SourceRange::new(6, 11));
        assert_eq!(second.get_value(), "world");
        assert_eq!(second.get_matched_value(), "world");
        assert_eq!(second.get_children().len(), 0);

        assert_eq!(
          first_child.get_attribute("hello"),
          Some(&"world".to_string())
        );

        assert_eq!(second.get_attribute("hello"), Some(&"world".to_string()));
      } else {
        unreachable!("Test failed!");
      };
    } else {
      unreachable!("Test failed!");
    };
  }

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
  fn it_can_capture_and_display_errors_properly() {
    if let Err(errors) =
      compile_script_from_file("./src/script/v1/tests/script/test_attribute_error1.axo", 0)
    {
      assert_eq!(errors.len(), 1);
      assert_eq!(errors[0].message, "Error: /home/wyatt/Projects/rust-adextopa-core/src/script/v1/tests/script/test_attribute_error1.axo@[1:32-37]: Malformed attribute detected. Attribute value is not single-quoted. The proper format for an attribute is: name='value'");
      assert_eq!(errors[0].range, Some(SourceRange::new(31, 36)));
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
