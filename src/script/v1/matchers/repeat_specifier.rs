use std::ops::Range;

use crate::token::TokenRef;

#[macro_export]
macro_rules! ScriptRepeatSpecifier {
  () => {
    $crate::Switch!("RepeatSpecifier";
      $crate::ScriptRepeatZeroOrMore!(),
      $crate::ScriptRepeatOneOrMore!(),
      $crate::ScriptRepeatRange!(),
    )
  };
}

pub fn get_repeat_specifier_range(token: TokenRef) -> Result<Range<usize>, String> {
  let token = token.borrow();
  let token_name = token.get_name();

  if token_name == "RepeatZeroOrMore" {
    Ok(0..usize::MAX)
  } else if token_name == "RepeatOneOrMore" {
    Ok(1..usize::MAX)
  } else if token_name == "RepeatRange" {
    // Here, we just unwrap, because a RepeatRange MUST have a "Minimum" child
    // If it doesn't... something is very wrong, and we should just panic
    let min = token.find_child("Minimum").unwrap();
    let has_seperator = token.has_child("Seperator");

    let minimum = match min.borrow().value().parse::<usize>() {
      Ok(result) => result,
      Err(error) => return Err(error.to_string()),
    };

    if has_seperator {
      match token.find_child("Maximum") {
        Some(ref max) => {
          let maximum = match max.borrow().value().parse::<usize>() {
            Ok(result) => result,
            Err(error) => return Err(error.to_string()),
          };

          if maximum < minimum {
            return Err(format!(
              "Error with repeat specifier: Maximum range is smaller than minimum: {{{}, {}}}",
              minimum, maximum
            ));
          }

          Ok(minimum..maximum)
        }
        None => Ok(minimum..usize::MAX),
      }
    } else {
      Ok(0..minimum)
    }
  } else {
    Err(format!("Expected one of `RepeatZeroOrMore`, `RepeatOneOrMore`, or `RepeatRange` tokens, but received a `{}` token instead", token_name))
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    script::current::matchers::repeat_specifier::get_repeat_specifier_range,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new(r"+*{}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      assert_eq!(get_repeat_specifier_range(token.clone()), Ok(1..usize::MAX));

      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatOneOrMore");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 1));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 1));
      assert_eq!(token.value(), r"+");
      assert_eq!(token.raw_value(), r"+");
      assert_eq!(token.get_children().len(), 0);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works2() {
    let parser = Parser::new(r"*+{}");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      assert_eq!(get_repeat_specifier_range(token.clone()), Ok(0..usize::MAX));

      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatZeroOrMore");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 1));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 1));
      assert_eq!(token.value(), r"*");
      assert_eq!(token.raw_value(), r"*");
      assert_eq!(token.get_children().len(), 0);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works3() {
    let parser = Parser::new(r"{10,}*+");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      assert_eq!(
        get_repeat_specifier_range(token.clone()),
        Ok(10..usize::MAX)
      );

      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 4));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 5));
      assert_eq!(token.value(), r"10,");
      assert_eq!(token.raw_value(), r"{10,}");
      assert_eq!(token.get_children().len(), 2);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_works4() {
    let parser = Parser::new(r"{10, 15}*+");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      assert_eq!(get_repeat_specifier_range(token.clone()), Ok(10..15));

      let token = token.borrow();
      assert_eq!(token.get_name(), "RepeatRange");
      assert_eq!(*token.get_captured_range(), SourceRange::new(1, 7));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 8));
      assert_eq!(token.value(), r"10, 15");
      assert_eq!(token.raw_value(), r"{10, 15}");
      assert_eq!(token.get_children().len(), 3);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new(r"{15, 10}*+");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(MatcherSuccess::Token(token)) = result {
      assert_eq!(
        get_repeat_specifier_range(token.clone()),
        Err(
          "Error with repeat specifier: Maximum range is smaller than minimum: {15, 10}"
            .to_string()
        )
      );
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails2() {
    let parser = Parser::new(" +");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptRepeatSpecifier!();

    if let Err(MatcherFailure::Fail) = ParserContext::tokenize(parser_context, matcher) {
    } else {
      unreachable!("Test failed!");
    };
  }
}
