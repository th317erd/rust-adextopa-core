extern crate adextopa_macros;

use crate::matcher::{Matcher, MatcherFailure, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef};
use std::ops::{Bound, Range, RangeBounds};

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

pub enum MatchAction {
  Continue,
  Stop,
}

pub struct ProgramPattern<'a> {
  patterns: Vec<Box<dyn Matcher>>,
  name: &'a str,
  pub(self) iterate_range: Option<Range<usize>>,
  pub(self) stop_on_first: MatchAction,
}

impl<'a> ProgramPattern<'a> {
  pub fn new_blank_program(stop_on_first: MatchAction) -> Self {
    let name = match stop_on_first {
      MatchAction::Stop => "Switch",
      MatchAction::Continue => "Program",
    };

    Self {
      patterns: Vec::new(),
      iterate_range: None,
      name,
      stop_on_first,
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
      stop_on_first: MatchAction::Continue,
    }
  }

  pub fn new_program(patterns: Vec<Box<dyn Matcher>>, stop_on_first: MatchAction) -> Self {
    let name = match stop_on_first {
      MatchAction::Stop => "Switch",
      MatchAction::Continue => "Program",
    };

    Self {
      patterns,
      iterate_range: None,
      name,
      stop_on_first,
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
      stop_on_first: MatchAction::Continue,
    }
  }

  pub fn new_program_with_name(
    patterns: Vec<Box<dyn Matcher>>,
    name: &'a str,
    stop_on_first: MatchAction,
  ) -> ProgramPattern<'a> {
    Self {
      patterns,
      name,
      iterate_range: None,
      stop_on_first,
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
      stop_on_first: MatchAction::Continue,
    }
  }

  pub fn add_pattern(&mut self, pattern: Box<dyn Matcher>) {
    self.patterns.push(pattern);
  }

  pub fn set_name(&mut self, name: &'a str) {
    self.name = name;
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
    program_token.set_children(children);
    program_token.set_value_range(value_range);
    program_token.set_raw_range(raw_range);
  }

  Ok(MatcherSuccess::Token(program_token))
}

fn add_token_to_children<'a>(
  program_token: &TokenRef<'a>,
  context: &ParserContextRef,
  children: &mut Vec<TokenRef<'a>>,
  value_range: &mut SourceRange,
  raw_range: &mut SourceRange,
  token: &TokenRef<'a>,
  assert_moving_forward: bool,
) {
  {
    if token.borrow().get_name() != "Error" {
      let token = token.borrow();

      if assert_moving_forward {
        // Ensure that we are moving forward, and that the token doesn't have a zero width
        assert!(token.get_raw_range().end != context.borrow().offset.start);
      }

      // value_range is set to raw_range because the program
      // should always span the range of all child tokens
      contain_source_range(value_range, &token.get_raw_range());
      contain_source_range(raw_range, &token.get_raw_range());
    } else {
      let mut token = token.borrow_mut();
      let sr = token.get_raw_range();

      if sr.start == usize::MAX || sr.end == usize::MAX {
        let mut source_range = SourceRange::new_blank();

        if value_range.start == usize::MAX {
          source_range.start = context.borrow().offset.start;
        } else {
          source_range.start = value_range.start;
        }

        source_range.end = context.borrow().offset.start;

        token.set_value_range(source_range);
        token.set_raw_range(source_range);
      }

      // value_range is set to raw_range because the program
      // should always span the range of all child tokens
      contain_source_range(value_range, &token.get_raw_range());
      contain_source_range(raw_range, &token.get_raw_range());
    }

    context
      .borrow_mut()
      .set_start(token.borrow().get_raw_range().end);
  }

  {
    let mut token = token.borrow_mut();
    token.set_parent(Some(program_token.clone()));
  }

  children.push(token.clone());
}

impl<'a> Matcher for ProgramPattern<'a> {
  fn exec(&self, context: ParserContextRef) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let mut sub_context = context.clone_with_name(self.get_name());
    let start_offset = sub_context.borrow().offset.start;
    let program_token = StandardToken::new(
      &sub_context.borrow().parser,
      self.name,
      SourceRange::new_blank(),
    );
    let mut children = Vec::<TokenRef>::with_capacity(self.patterns.len());
    let mut value_range = SourceRange::new(usize::MAX, 0);
    let mut raw_range = SourceRange::new(usize::MAX, 0);
    let is_loop = match &self.iterate_range {
      Some(_) => true,
      None => false,
    };
    let iterate_range = match &self.iterate_range {
      Some(range) => range.clone(),
      None => (0..1),
    };
    let mut iteration_result: Option<MatcherSuccess>;

    for _ in iterate_range {
      iteration_result = None;

      for pattern in &self.patterns {
        let result = pattern.exec(std::rc::Rc::new(std::cell::RefCell::new(
          sub_context.borrow().clone(),
        )));

        match result {
          Ok(success) => match success {
            MatcherSuccess::Token(token) => {
              match self.stop_on_first {
                MatchAction::Stop => {
                  return Ok(MatcherSuccess::Token(token.clone()));
                }
                _ => {}
              }

              if sub_context.borrow().debug_mode_level() > 1 {
                let token = token.borrow();

                if sub_context.borrow().debug_mode_level() > 2 {
                  print!("{{Token}} ");
                }

                println!(
                  "'{}' Adding child '{}' @[{}-{}]",
                  self.get_name(),
                  token.get_name(),
                  token.get_raw_range().start,
                  token.get_raw_range().end
                );
              }

              add_token_to_children(
                &program_token,
                &sub_context,
                &mut children,
                &mut value_range,
                &mut raw_range,
                &token,
                true,
              );

              if sub_context.borrow().is_debug_mode() {
                if sub_context.borrow().debug_mode_level() > 2 {
                  print!("{{Token}} ");
                }

                println!(
                  "'{}' Setting to offset to: {} -> {}",
                  self.get_name(),
                  sub_context.borrow().offset.start,
                  token.borrow().get_raw_range().end
                );
              }
            }
            MatcherSuccess::ExtractChildren(token) => {
              match self.stop_on_first {
                MatchAction::Stop => {
                  return Ok(MatcherSuccess::ExtractChildren(token.clone()));
                }
                _ => {}
              }

              let token = token.borrow();
              let target_children = token.get_children();

              if sub_context.borrow().is_debug_mode() {
                if sub_context.borrow().debug_mode_level() > 2 {
                  print!("{{ExtractChildren}} ");
                }

                println!(
                  "'{}' Setting to offset to: {} -> {}",
                  self.get_name(),
                  sub_context.borrow().offset.start,
                  token.get_raw_range().end
                );
              }

              sub_context
                .borrow_mut()
                .set_start(token.get_raw_range().end);

              contain_source_range(&mut value_range, &token.get_raw_range());
              contain_source_range(&mut raw_range, &token.get_raw_range());

              if sub_context.borrow().debug_mode_level() > 1 {
                if sub_context.borrow().debug_mode_level() > 2 {
                  print!("{{ExtractChildren}} ");
                }

                let count = target_children.len();
                println!(
                  "'{}' Will be adding {} {}",
                  self.get_name(),
                  count,
                  if count != 1 { "children" } else { "child" }
                );
              }

              for child in target_children {
                if sub_context.borrow().debug_mode_level() > 1 {
                  let child = child.borrow();

                  if sub_context.borrow().debug_mode_level() > 2 {
                    print!("{{ExtractChildren}} ");
                  }

                  println!(
                    "'{}' Adding child '{}' @[{}-{}]",
                    self.get_name(),
                    child.get_name(),
                    child.get_raw_range().start,
                    child.get_raw_range().end
                  );
                }

                add_token_to_children(
                  &program_token,
                  &sub_context,
                  &mut children,
                  &mut value_range,
                  &mut raw_range,
                  &child,
                  false,
                );
              }
            }
            MatcherSuccess::Skip(amount) => {
              let new_offset = sub_context.borrow().offset.start + amount as usize;

              if sub_context.borrow().is_debug_mode() {
                if sub_context.borrow().debug_mode_level() > 2 {
                  print!("{{Skip}} ");
                }

                println!(
                  "'{}' Skipping: {} + {} -> {}",
                  self.get_name(),
                  sub_context.borrow().offset.start,
                  amount,
                  new_offset
                );
              }

              sub_context.borrow_mut().set_start(new_offset);

              let range = SourceRange::new(start_offset, new_offset);

              contain_source_range(&mut value_range, &range);
              contain_source_range(&mut raw_range, &range);

              continue;
            }
            _ => {
              iteration_result = Some(success);
              break;
            }
          },
          Err(failure) => {
            let sub_context = sub_context.borrow();
            if sub_context.is_debug_mode() {
              if sub_context.debug_mode_level() > 2 {
                print!("{{Failure}} ");
              }

              println!(
                "'{}' failure! -->|{}|--> @[{}-{}]",
                self.get_name(),
                sub_context.debug_range(10),
                sub_context.offset.start,
                sub_context.offset.end
              );
            }

            if is_loop {
              return finalize_program_token(program_token, children, value_range, raw_range);
            } else {
              match failure {
                MatcherFailure::Fail => match self.stop_on_first {
                  MatchAction::Stop => {
                    continue;
                  }
                  _ => {
                    return Err(MatcherFailure::Fail);
                  }
                },
                MatcherFailure::Error(error) => {
                  return Err(MatcherFailure::Error(error));
                }
              }
            }
          }
        }
      }

      match iteration_result {
        Some(action) => match action {
          MatcherSuccess::Break((loop_name, data)) => {
            if is_loop && (loop_name == self.name || loop_name == "") {
              match &*data {
                MatcherSuccess::Token(token) => {
                  if sub_context.borrow().debug_mode_level() > 1 {
                    let token = token.borrow();

                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Break/Token}} ");
                    }

                    println!(
                      "'{}' Adding child '{}' @[{}-{}]",
                      self.get_name(),
                      token.get_name(),
                      token.get_raw_range().start,
                      token.get_raw_range().end
                    );
                  }

                  // Add token to myself, and then continue propagating
                  add_token_to_children(
                    &program_token,
                    &mut sub_context,
                    &mut children,
                    &mut value_range,
                    &mut raw_range,
                    &token,
                    true,
                  );

                  if sub_context.borrow().is_debug_mode() {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Break/Token}} ");
                    }

                    println!(
                      "'{}' Setting to offset to: {} -> {}",
                      self.get_name(),
                      sub_context.borrow().offset.start,
                      token.borrow().get_raw_range().end
                    );
                  }

                  Box::new(MatcherSuccess::None)
                }
                MatcherSuccess::ExtractChildren(token) => {
                  let token = token.borrow();
                  let target_children = token.get_children();

                  if sub_context.borrow().is_debug_mode() {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Break/ExtractChildren}} ");
                    }

                    println!(
                      "'{}' Setting to offset to: {} -> {}",
                      self.get_name(),
                      sub_context.borrow().offset.start,
                      token.get_raw_range().end
                    );
                  }

                  sub_context
                    .borrow_mut()
                    .set_start(token.get_raw_range().end);

                  contain_source_range(&mut value_range, &token.get_raw_range());
                  contain_source_range(&mut raw_range, &token.get_raw_range());

                  if sub_context.borrow().debug_mode_level() > 1 {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Break/ExtractChildren}} ");
                    }

                    let count = target_children.len();
                    println!(
                      "'{}' Will be adding {} {}",
                      self.get_name(),
                      count,
                      if count != 1 { "children" } else { "child" }
                    );
                  }

                  for child in target_children {
                    add_token_to_children(
                      &program_token,
                      &sub_context,
                      &mut children,
                      &mut value_range,
                      &mut raw_range,
                      &child,
                      false,
                    );

                    if sub_context.borrow().is_debug_mode() {
                      let child = child.borrow();

                      if sub_context.borrow().debug_mode_level() > 2 {
                        print!("{{Break/ExtractChildren}} ");
                      }

                      println!(
                        "'{}' Adding child '{}' @[{}-{}]",
                        self.get_name(),
                        child.get_name(),
                        child.get_raw_range().start,
                        child.get_raw_range().end
                      );
                    }
                  }

                  Box::new(MatcherSuccess::None)
                }
                _ => data,
              };

              // This is the loop that should break, so cease propagating the Break
              return finalize_program_token(program_token, children, value_range, raw_range);
            } else {
              // if children.len() == 0 {
              //   return Ok(MatcherSuccess::Break((loop_name, data)));
              // }

              match finalize_program_token(program_token, children, value_range, raw_range) {
                Ok(final_token) => {
                  return Ok(MatcherSuccess::Break((loop_name, Box::new(final_token))));
                }
                Err(_) => {
                  return Ok(MatcherSuccess::Break((loop_name, data)));
                }
              }
            }
          }
          MatcherSuccess::Continue((loop_name, data)) => {
            // This is not the correct Loop, or is a Program, so propagate Continue
            if is_loop && (loop_name == self.name || loop_name == "") {
              match &*data {
                MatcherSuccess::Token(token) => {
                  if sub_context.borrow().debug_mode_level() > 1 {
                    let token = token.borrow();

                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Continue/Token}} ");
                    }

                    println!(
                      "'{}' Adding child '{}' @[{}-{}]",
                      self.get_name(),
                      token.get_name(),
                      token.get_raw_range().start,
                      token.get_raw_range().end,
                    );
                  }

                  // Add token to myself, and then continue propagating
                  add_token_to_children(
                    &program_token,
                    &mut sub_context,
                    &mut children,
                    &mut value_range,
                    &mut raw_range,
                    &token,
                    true,
                  );

                  if sub_context.borrow().is_debug_mode() {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Continue/Token}} ");
                    }

                    println!(
                      "'{}' Setting to offset to: {} -> {}",
                      self.get_name(),
                      sub_context.borrow().offset.start,
                      token.borrow().get_raw_range().end
                    );
                  }

                  Box::new(MatcherSuccess::None)
                }
                MatcherSuccess::ExtractChildren(token) => {
                  let token = token.borrow();
                  let target_children = token.get_children();

                  if sub_context.borrow().is_debug_mode() {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Continue/ExtractChildren}} ");
                    }

                    println!(
                      "'{}' Setting to offset to: {} -> {}",
                      self.get_name(),
                      sub_context.borrow().offset.start,
                      token.get_raw_range().end
                    );
                  }

                  sub_context
                    .borrow_mut()
                    .set_start(token.get_raw_range().end);

                  contain_source_range(&mut value_range, &token.get_raw_range());
                  contain_source_range(&mut raw_range, &token.get_raw_range());

                  if sub_context.borrow().debug_mode_level() > 1 {
                    if sub_context.borrow().debug_mode_level() > 2 {
                      print!("{{Continue/ExtractChildren}} ");
                    }

                    let count = target_children.len();
                    println!(
                      "'{}' Will be adding {} {}",
                      self.get_name(),
                      count,
                      if count != 1 { "children" } else { "child" }
                    );
                  }

                  for child in target_children {
                    add_token_to_children(
                      &program_token,
                      &sub_context,
                      &mut children,
                      &mut value_range,
                      &mut raw_range,
                      &child,
                      false,
                    );

                    if sub_context.borrow().debug_mode_level() > 1 {
                      let child = child.borrow();

                      if sub_context.borrow().debug_mode_level() > 2 {
                        print!("{{Continue/ExtractChildren}} ");
                      }

                      println!(
                        "'{}' Adding child '{}' @[{}-{}]",
                        self.get_name(),
                        child.get_name(),
                        child.get_raw_range().start,
                        child.get_raw_range().end
                      );
                    }
                  }

                  Box::new(MatcherSuccess::None)
                }
                _ => data,
              };
            } else {
              // if children.len() == 0 {
              //   return Ok(MatcherSuccess::Continue((loop_name, data)));
              // }

              match finalize_program_token(program_token, children, value_range, raw_range) {
                Ok(final_token) => {
                  return Ok(MatcherSuccess::Continue((loop_name, Box::new(final_token))));
                }
                Err(_) => {
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
      let mut program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Continue);
      program.set_name($name);

      $(
        program.add_pattern(std::boxed::Box::new($args));
      )*

      program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let mut program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Continue);
      $(
        program.add_pattern(std::boxed::Box::new($args));
      )*

      program
    }
  };
}

#[macro_export]
macro_rules! Switch {
  ($name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Stop);
      program.set_name($name);

      $(
        program.add_pattern(std::boxed::Box::new($args));
      )*

      program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let mut program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Stop);

      $(
        program.add_pattern(std::boxed::Box::new($args));
      )*

      program
    }
  };
}

#[macro_export]
macro_rules! Loop {
  ($range:expr; $name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop($range);
      loop_program.set_name($name);

      $(
        loop_program.add_pattern(std::boxed::Box::new($args));
      )*

      loop_program
    }
  };

  ($name:expr; $($args:expr),+ $(,)?) => {
    {
      let mut loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop(0..);
      loop_program.set_name($name);

      $(
        loop_program.add_pattern(std::boxed::Box::new($args));
      )*

      loop_program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let mut loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop(0..);

      $(
        loop_program.add_pattern(std::boxed::Box::new($args));
      )*

      loop_program
    }
  };
}

#[cfg(test)]
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
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Equals!("Testing"), Equals!(" "), Matches!(r"\d+"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), parser.borrow().source);
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails_matching_against_a_simple_program() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Equals!("Testing"), Matches!(r"\d+"));

    assert_eq!(
      Err(MatcherFailure::Fail),
      matcher.exec(parser_context.clone())
    );
  }

  #[test]
  fn it_matches_against_a_simple_switch() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Switch!(Equals!(" "), Matches!(r"\d+"), Equals!("Testing"));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 7));
      assert_eq!(token.value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_matches_against_a_loop() {
    let parser = Parser::new("A B C D E F ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(Matches!(r"\w"), Equals!(" "));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), parser.borrow().source);

      assert_eq!(token.get_children().len(), 12);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        assert_eq!(
          token.get_children()[index * 2].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          token.get_children()[index * 2].borrow().value(),
          parts[index]
        );
        assert_eq!(
          token.get_children()[index * 2 + 1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(token.get_children()[index * 2 + 1].borrow().value(), " ");
      }
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_matches_against_a_loop_with_a_program() {
    let parser = Parser::new("A B C D E F ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(Program!(Matches!(r"\w"), Equals!(" ")));

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 12));
      assert_eq!(token.value(), parser.borrow().source);

      assert_eq!(token.get_children().len(), 6);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        let program_token = &token.get_children()[index];

        assert_eq!(
          program_token.borrow().get_children()[0].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          program_token.borrow().get_children()[0].borrow().value(),
          parts[index]
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().value(),
          " "
        );
      }
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_can_break_from_a_loop() {
    let parser = Parser::new("A B C break D E F ");
    let parser_context = ParserContext::new(&parser, "Test");
    let capture = Program!(Matches!(r"\w"), Equals!(" "));
    let brk = Optional!(Program!(Equals!("break"), Break!()));
    let matcher = Loop!(0..10; "Loop"; brk, capture);

    if let Ok(MatcherSuccess::Token(token)) = matcher.exec(parser_context.clone()) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_value_range(), SourceRange::new(0, 11));
      assert_eq!(token.value(), "A B C break");

      assert_eq!(token.get_children().len(), 4);

      let parts = vec!["A", "B", "C"];

      for index in 0..parts.len() {
        let program_token = &token.get_children()[index];

        assert_eq!(
          program_token.borrow().get_children()[0].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          program_token.borrow().get_children()[0].borrow().value(),
          parts[index]
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().value(),
          " "
        );
      }

      let program_token = &token.get_children()[3];
      assert_eq!(program_token.borrow().get_name(), "Program");
      assert_eq!(program_token.borrow().get_children().len(), 1);
      assert_eq!(
        program_token.borrow().get_children()[0].borrow().get_name(),
        "Equals"
      );
      assert_eq!(
        program_token.borrow().get_children()[0].borrow().value(),
        "break"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
