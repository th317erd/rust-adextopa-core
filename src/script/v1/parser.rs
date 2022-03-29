use std::collections::HashMap;

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

fn construct_matcher_from_pattern_definition<'a>(
  parser_context: ParserContextRef<'a>,
  token: TokenRef,
  name: &str,
  captured: bool,
) -> Result<(MatcherRef<'a>, MatcherRef<'a>), String> {
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
        let key = child_children[0].borrow().value();
        let value = child_children[1].borrow().value();

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

fn construct_matcher_from_pattern<'a>(
  parser_context: ParserContextRef<'a>,
  token: TokenRef,
) -> Result<(MatcherRef<'a>, MatcherRef<'a>), String> {
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
  name: String,
) -> Result<MatcherRef<'a>, String> {
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
      let matcher_name = identifier.value();
      let value = &token_children[1];
      let _value = value.borrow();
      let value_name = _value.get_name();

      if value_name == "Identifier" {
        // Identifier is assigned to identifier...
        // so this is a reference
        register_matchers.borrow_mut().add_pattern(crate::Ref!(matcher_name.clone(); _value.value()));
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

pub fn compile_script<'a>(parser: ParserRef, name: String) -> Result<MatcherRef<'a>, String> {
  let parser_context = ParserContext::new(&parser, &name);

  (*parser_context)
    .borrow()
    .register_matchers(None, vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);

  let pattern = crate::Script!();

  let result = ParserContext::tokenize(parser_context.clone(), pattern);
  match result {
    Ok(result) => match result {
      MatcherSuccess::Token(token) => {
        build_matcher_from_tokens(token.clone(), parser_context.clone(), name)
      }
      MatcherSuccess::ExtractChildren(token) => {
        build_matcher_from_tokens(token.clone(), parser_context.clone(), name)
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

pub fn compile_script_from_str<'a>(
  source: &'a str,
  name: String,
) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new(source);
  compile_script(parser, name)
}

pub fn compile_script_from_file<'a>(file_name: &'a str) -> Result<MatcherRef<'a>, String> {
  let parser = Parser::new_from_file(file_name).unwrap();
  compile_script(parser, file_name.to_string())
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherSuccess,
    parser::Parser,
    parser_context::{ParserContext, ParserContextRef},
    script::current::parser::construct_matcher_from_pattern,
    source_range::SourceRange,
    Debug, ScriptPattern, ScriptPatternDefinition, ScriptProgramMatcher, ScriptSwitchMatcher,
  };

  use super::{compile_script_from_file, construct_matcher_from_pattern_definition};

  #[test]
  fn it_compiles_a_script_and_returns_a_matcher() {
    if let Ok(compiled_matcher) =
      compile_script_from_file("./src/script/v1/tests/script/test_word.axo")
    {
      let parser = Parser::new("test");
      let parser_context = ParserContext::new(&parser, "Test");

      println!("MATCHER: {:?}", compiled_matcher);
      let compiled_matcher = Debug!(compiled_matcher);

      let result = ParserContext::tokenize(parser_context, compiled_matcher.clone());

      if let Ok(MatcherSuccess::Token(token)) = result {
        let token = token.borrow();

        assert_eq!(
          token.get_name(),
          "./src/script/v1/tests/script/test_word.axo"
        );
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 1));
        assert_eq!(*token.get_matched_range(), SourceRange::new(0, 1));
        assert_eq!(token.value(), "test");
        assert_eq!(token.raw_value(), "test");
        assert_eq!(token.get_children().len(), 1);

        let first = token.get_children()[0].borrow();
        assert_eq!(first.get_name(), "Identifier");
        assert_eq!(*first.get_captured_range(), SourceRange::new(0, 4));
        assert_eq!(*first.get_matched_range(), SourceRange::new(0, 4));
        assert_eq!(first.value(), "test");
        assert_eq!(first.raw_value(), "test");

        assert_eq!(first.get_attribute("hello"), Some(&"world".to_string()));
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
      .register_matchers(None, vec![ScriptSwitchMatcher!(), ScriptProgramMatcher!()]);
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

    if let Ok(MatcherSuccess::Token(token)) = result {
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

      if let Ok(MatcherSuccess::Token(token)) = result2 {
        let token = token.borrow();
        assert_eq!(token.get_name(), "Matches");
        assert_eq!(*token.get_captured_range(), SourceRange::new(0, 4));
        assert_eq!(*token.get_matched_range(), SourceRange::new(0, 4));
        assert_eq!(token.value(), "test");
        assert_eq!(token.raw_value(), "test");

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
