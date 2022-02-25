use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeTo};

use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContext;
use crate::source_range::SourceRange;
use crate::token::{Token, TokenRef};

fn get_range<T>(r: T) -> Range<usize>
where
  T: RangeBounds<usize>,
{
  let mut range = 0..usize::MAX;

  if let Bound::Included(start) = r.start_bound() {
    range.start = *start;
  }

  if let Bound::Excluded(end) = r.end_bound() {
    range.start = *end;
  }

  range
}

pub struct ProgramPattern<'a> {
  patterns: Vec<Box<dyn Matcher>>,
  name: &'a str,
  pub(self) iterate_range: Option<Range<usize>>,
}

impl<'a> ProgramPattern<'a> {
  pub fn new_blank_program() -> Self {
    Self {
      patterns: Vec::new(),
      name: "Program",
      iterate_range: None,
    }
  }

  pub fn new_blank_loop<T>(r: T) -> Self
  where
    T: RangeBounds<usize>,
  {
    Self {
      patterns: Vec::new(),
      name: "Loop",
      iterate_range: Some(get_range(r)),
    }
  }

  pub fn new_program(patterns: Vec<Box<dyn Matcher>>) -> Self {
    Self {
      patterns,
      name: "Program",
      iterate_range: None,
    }
  }

  pub fn new_loop<T>(patterns: Vec<Box<dyn Matcher>>, r: T) -> Self
  where
    T: RangeBounds<usize>,
  {
    Self {
      patterns,
      name: "Loop",
      iterate_range: Some(get_range(r)),
    }
  }

  pub fn new_program_with_name(
    patterns: Vec<Box<dyn Matcher>>,
    name: &'a str,
  ) -> ProgramPattern<'a> {
    Self {
      patterns,
      name,
      iterate_range: None,
    }
  }

  pub fn new_loop_with_name<T>(
    patterns: Vec<Box<dyn Matcher>>,
    name: &'a str,
    r: T,
  ) -> ProgramPattern<'a>
  where
    T: RangeBounds<usize>,
  {
    Self {
      patterns,
      name,
      iterate_range: Some(get_range(r)),
    }
  }
}

fn contain_source_range(tsr: &mut SourceRange, ssr: &SourceRange) {
  if ssr.start < tsr.start {
    tsr.start = ssr.start;
  }

  if ssr.end > tsr.end {
    tsr.end = ssr.end;
  }
}

fn finalize_program_token<'a>(
  program_token: TokenRef<'a>,
  children: Vec<TokenRef<'a>>,
  value_range: SourceRange,
  raw_range: SourceRange,
) -> Result<MatcherSuccess<'a>, MatcherFailure<'a>> {
  if value_range.start == usize::MAX || raw_range.start == usize::MAX {
    return Err(MatcherFailure::Fail);
  }

  {
    let mut program_token = program_token.borrow_mut();
    program_token.children = children;
    program_token.value_range = value_range;
    program_token.raw_range = raw_range;
  }

  Ok(MatcherSuccess::Token(program_token))
}

fn add_token_to_children<'a>(
  program_token: &TokenRef<'a>,
  context: &mut ParserContext,
  children: &mut Vec<TokenRef<'a>>,
  value_range: &mut SourceRange,
  raw_range: &mut SourceRange,
  token: &TokenRef<'a>,
) {
  {
    let token = token.borrow();
    // Ensure that we are moving forward, and that the token doesn't have a zero width
    assert!(token.raw_range.end != context.offset.start);

    contain_source_range(value_range, &token.value_range);
    contain_source_range(raw_range, &token.raw_range);

    context.set_start(token.raw_range.end);
  }

  {
    token.borrow_mut().parent = Some(program_token.clone());
  }

  children.push(token.clone());
}

impl<'a> Matcher for ProgramPattern<'a> {
  fn exec(&self, context: &ParserContext) -> Result<MatcherSuccess, MatcherFailure> {
    let mut sub_context = context.clone();
    let program_token = Token::new(self.name, SourceRange::new_blank());
    let mut children = Vec::<TokenRef>::with_capacity(self.patterns.len());
    let mut value_range = SourceRange::new(usize::MAX, 0);
    let mut raw_range = SourceRange::new(usize::MAX, 0);
    let is_loop = match &self.iterate_range {
      Some(_) => true,
      None => false,
    };
    let mut iterate_range = match &self.iterate_range {
      Some(range) => range.clone(),
      None => (0..1),
    };
    let mut iteration_result: Option<MatcherSuccess> = None;

    for index in iterate_range {
      iteration_result = None;

      for pattern in &self.patterns {
        let result = pattern.exec(&sub_context);
        match result {
          Ok(success) => match success {
            MatcherSuccess::Token(token) => {
              add_token_to_children(
                &program_token,
                &mut sub_context,
                &mut children,
                &mut value_range,
                &mut raw_range,
                &token,
              );
            }
            MatcherSuccess::Skip(amount) => {
              sub_context.set_start(sub_context.offset.start + amount as usize);
              continue;
            }
            _ => {
              iteration_result = Some(success);
              break;
            }
          },
          Err(failure) => {
            if is_loop {
              return finalize_program_token(program_token, children, value_range, raw_range);
            } else {
              return Err(failure);
            }
          }
        }
      }

      match iteration_result {
        Some(action) => match action {
          MatcherSuccess::Break((loop_name, data)) => {
            let data = match &*data {
              MatcherSuccess::Token(token) => {
                // Add token to myself, and then continue propagating
                add_token_to_children(
                  &program_token,
                  &mut sub_context,
                  &mut children,
                  &mut value_range,
                  &mut raw_range,
                  &token,
                );

                Box::new(MatcherSuccess::None)
              }
              _ => data,
            };

            if is_loop && (loop_name == self.name || loop_name == "") {
              // This is the loop that should break, so cease propagating the Break
              return finalize_program_token(program_token, children, value_range, raw_range);
            } else {
              if children.len() == 0 {
                return Ok(MatcherSuccess::Break((loop_name, data)));
              }

              match finalize_program_token(program_token, children, value_range, raw_range) {
                Ok(final_token) => {
                  return Ok(MatcherSuccess::Break((loop_name, Box::new(final_token))));
                }
                Err(failure) => {
                  return Ok(MatcherSuccess::Break((loop_name, data)));
                }
              }
            }
          }
          MatcherSuccess::Continue((loop_name, data)) => {
            let data = match &*data {
              MatcherSuccess::Token(token) => {
                // Add token to myself, and then continue propagating
                add_token_to_children(
                  &program_token,
                  &mut sub_context,
                  &mut children,
                  &mut value_range,
                  &mut raw_range,
                  &token,
                );

                Box::new(MatcherSuccess::None)
              }
              _ => data,
            };

            // This is not the correct Loop, or is a Program, so propagate Continue
            if !(is_loop && (loop_name == self.name || loop_name == "")) {
              if children.len() == 0 {
                return Ok(MatcherSuccess::Continue((loop_name, data)));
              }

              match finalize_program_token(program_token, children, value_range, raw_range) {
                Ok(final_token) => {
                  return Ok(MatcherSuccess::Continue((loop_name, Box::new(final_token))));
                }
                Err(failure) => {
                  return Ok(MatcherSuccess::Continue((loop_name, data)));
                }
              }
            }
          }
          MatcherSuccess::Stop => {
            break;
          }
          _ => unreachable!(),
        },
        None => continue,
      }
    }

    finalize_program_token(program_token, children, value_range, raw_range)
  }

  fn get_name(&self) -> &str {
    self.name
  }
}

#[macro_export]
macro_rules! Program {
  ($name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut program = crate::matchers::program::ProgramPattern::new_blank_program();
      program.name = $name;

      $(
        program.patterns.push(Box::new($args));
      )*
      program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let mut program = crate::matchers::program::ProgramPattern::new_blank_program();
      $(
        program.patterns.push(Box::new($args));
      )*
      program
    }
  }
}

#[macro_export]
macro_rules! Loop {
  ($range:expr; $name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut loop_program = crate::matchers::program::ProgramPattern::new_blank_loop($range);
      loop_program.name = $name;

      $(
        loop_program.patterns.push(Box::new($args));
      )*
      loop_program
    }
  };

  ($name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut loop_program = crate::matchers::program::ProgramPattern::new_blank_loop(0..);
      loop_program.name = $name;

      $(
        loop_program.patterns.push(Box::new($args));
      )*
      loop_program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let mut loop_program = crate::matchers::program::ProgramPattern::new_blank_loop(0..);

      $(
        loop_program.patterns.push(Box::new($args));
      )*
      loop_program
    }
  };
}

mod tests {
  use crate::{
    matcher::{Matcher, MatcherFailure, MatcherSuccess},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
    Break, Equals, Matches, Optional,
  };

  #[test]
  fn it_matches_against_a_simple_program() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Program!(Equals!("Testing"), Equals!(" "), Matches!(r"\d+"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Program");
      assert_eq!(token.value_range, SourceRange::new(0, 12));
      assert_eq!(token.value(&parser), parser.source);
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_fails_matching_against_a_simple_program() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser);
    let matcher = Program!(Equals!("Testing"), Matches!(r"\d+"));

    assert_eq!(matcher.exec(&parser_context), Err(MatcherFailure::Fail));
  }

  #[test]
  fn it_matches_against_a_loop() {
    let parser = Parser::new("A B C D E F ");
    let parser_context = ParserContext::new(&parser);
    let matcher = Loop!(Matches!(r"\w"), Equals!(" "));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Loop");
      assert_eq!(token.value_range, SourceRange::new(0, 12));
      assert_eq!(token.value(&parser), parser.source);

      assert_eq!(token.children.len(), 12);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        assert_eq!(token.children[index * 2].borrow().name, "Matches");
        assert_eq!(
          token.children[index * 2].borrow().value(&parser),
          parts[index]
        );
        assert_eq!(token.children[index * 2 + 1].borrow().name, "Equals");
        assert_eq!(token.children[index * 2 + 1].borrow().value(&parser), " ");
      }
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_matches_against_a_loop_with_a_program() {
    let parser = Parser::new("A B C D E F ");
    let parser_context = ParserContext::new(&parser);
    let matcher = Loop!(Program!(Matches!(r"\w"), Equals!(" ")));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Loop");
      assert_eq!(token.value_range, SourceRange::new(0, 12));
      assert_eq!(token.value(&parser), parser.source);

      assert_eq!(token.children.len(), 6);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        let program_token = &token.children[index];

        assert_eq!(program_token.borrow().children[0].borrow().name, "Matches");
        assert_eq!(
          program_token.borrow().children[0].borrow().value(&parser),
          parts[index]
        );
        assert_eq!(program_token.borrow().children[1].borrow().name, "Equals");
        assert_eq!(
          program_token.borrow().children[1].borrow().value(&parser),
          " "
        );
      }
    } else {
      unreachable!("Test failed!");
    }
  }

  #[test]
  fn it_can_break_from_a_loop() {
    let parser = Parser::new("A B C break D E F ");
    let parser_context = ParserContext::new(&parser);
    let capture = Program!(Matches!(r"\w"), Equals!(" "));
    let brk = Optional!(Program!(Equals!("break"), Break!()));
    let matcher = Loop!(0..10; "Loop"; brk, capture);

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(&parser_context) {
      let token = token.borrow();
      assert_eq!(token.name, "Loop");
      assert_eq!(token.value_range, SourceRange::new(0, 11));
      assert_eq!(token.value(&parser), "A B C break");

      assert_eq!(token.children.len(), 4);

      let parts = vec!["A", "B", "C"];

      for index in 0..parts.len() {
        let program_token = &token.children[index];

        assert_eq!(program_token.borrow().children[0].borrow().name, "Matches");
        assert_eq!(
          program_token.borrow().children[0].borrow().value(&parser),
          parts[index]
        );
        assert_eq!(program_token.borrow().children[1].borrow().name, "Equals");
        assert_eq!(
          program_token.borrow().children[1].borrow().value(&parser),
          " "
        );
      }

      let program_token = &token.children[3];
      assert_eq!(program_token.borrow().name, "Program");
      assert_eq!(program_token.borrow().children.len(), 1);
      assert_eq!(program_token.borrow().children[0].borrow().name, "Equals");
      assert_eq!(
        program_token.borrow().children[0].borrow().value(&parser),
        "break"
      );
    } else {
      unreachable!("Test failed!");
    }
  }
}
