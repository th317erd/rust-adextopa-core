extern crate adextopa_macros;

use crate::matcher::{Matcher, MatcherFailure, MatcherRef, MatcherSuccess};
use crate::parser_context::ParserContextRef;
use crate::scope_context::ScopeContextRef;
use crate::source_range::SourceRange;
use crate::token::{StandardToken, TokenRef, IS_ERROR};
use std::cell::RefCell;
use std::ops::{Bound, Range, RangeBounds};
use std::rc::Rc;

fn get_range<T>(r: T) -> Range<usize>
where
  T: RangeBounds<usize>,
{
  let mut range = 0..usize::MAX;

  if let Bound::Included(start) = r.start_bound() {
    range.start = *start;
  }

  if let Bound::Excluded(end) = r.end_bound() {
    range.end = *end;
  }

  range
}

#[derive(Debug)]
pub enum MatchAction {
  Continue,
  Stop,
}

pub struct ProgramPattern {
  patterns: Vec<MatcherRef>,
  name: String,
  pub(self) iterate_range: Option<Range<usize>>,
  pub(self) on_first_match: MatchAction,
  custom_name: bool,
}

impl std::fmt::Debug for ProgramPattern {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name: &str;

    if self.iterate_range.is_some() {
      name = "LoopPattern";
    } else if let MatchAction::Stop = self.on_first_match {
      name = "SwitchPattern";
    } else {
      name = "ProgramPattern";
    }

    f.debug_struct(name)
      .field("patterns", &self.patterns)
      .field("name", &self.name)
      .field("iterate_range", &self.iterate_range)
      .field("on_first_match", &self.on_first_match)
      .field("custom_name", &self.custom_name)
      .finish()
  }
}

impl ProgramPattern {
  pub fn new_blank_program(on_first_match: MatchAction) -> MatcherRef {
    let name = match on_first_match {
      MatchAction::Stop => "Switch",
      MatchAction::Continue => "Program",
    };

    Rc::new(RefCell::new(Box::new(Self {
      patterns: Vec::new(),
      iterate_range: None,
      name: name.to_string(),
      on_first_match,
      custom_name: false,
    })))
  }

  pub fn new_blank_loop<T>(r: T) -> MatcherRef
  where
    T: RangeBounds<usize>,
  {
    Rc::new(RefCell::new(Box::new(Self {
      patterns: Vec::new(),
      name: "Loop".to_string(),
      iterate_range: Some(get_range(r)),
      on_first_match: MatchAction::Continue,
      custom_name: false,
    })))
  }

  pub fn new_program(patterns: Vec<MatcherRef>, on_first_match: MatchAction) -> MatcherRef {
    let name = match on_first_match {
      MatchAction::Stop => "Switch",
      MatchAction::Continue => "Program",
    };

    Rc::new(RefCell::new(Box::new(Self {
      patterns,
      iterate_range: None,
      name: name.to_string(),
      on_first_match,
      custom_name: false,
    })))
  }

  pub fn new_loop<T>(patterns: Vec<MatcherRef>, r: T) -> MatcherRef
  where
    T: RangeBounds<usize>,
  {
    Rc::new(RefCell::new(Box::new(Self {
      patterns,
      name: "Loop".to_string(),
      iterate_range: Some(get_range(r)),
      on_first_match: MatchAction::Continue,
      custom_name: false,
    })))
  }

  pub fn new_program_with_name(
    patterns: Vec<MatcherRef>,
    name: String,
    on_first_match: MatchAction,
  ) -> MatcherRef {
    Rc::new(RefCell::new(Box::new(Self {
      patterns,
      name,
      iterate_range: None,
      on_first_match,
      custom_name: true,
    })))
  }

  pub fn new_loop_with_name_and_range<T>(
    patterns: Vec<MatcherRef>,
    name: String,
    r: T,
  ) -> MatcherRef
  where
    T: RangeBounds<usize>,
  {
    Rc::new(RefCell::new(Box::new(Self {
      patterns,
      name,
      iterate_range: Some(get_range(r)),
      on_first_match: MatchAction::Continue,
      custom_name: true,
    })))
  }

  fn _exec(
    &self,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    let context = context.borrow();
    let sub_context = context.clone_with_name(self.get_name());
    let start_offset = sub_context.borrow().offset.start;
    let program_token = StandardToken::new(
      &sub_context.borrow().parser,
      self.name.to_string(),
      SourceRange::new_blank(),
    );
    let mut children = Vec::<TokenRef>::with_capacity(self.patterns.len());
    let mut captured_range = SourceRange::new(usize::MAX, 0);
    let mut matched_range = SourceRange::new(usize::MAX, 0);
    let is_loop = match &self.iterate_range {
      Some(_) => true,
      None => false,
    };
    let iterate_range = match &self.iterate_range {
      Some(range) => range.clone(),
      None => (0..1),
    };
    let mut iteration_result: Option<MatcherSuccess>;
    let mut loop_count = 0;

    for _ in iterate_range {
      iteration_result = None;

      if sub_context.borrow().debug_mode_level() > 1 {
        println!("{{{}/Iterating}}", program_token.borrow().get_name());
      }

      for pattern in &self.patterns {
        sub_context
          .borrow_mut()
          .push_token_to_stack(program_token.clone());

        let result = pattern.borrow().exec(
          pattern.clone(),
          std::rc::Rc::new(std::cell::RefCell::new(sub_context.borrow().clone())),
          scope.clone(),
        );

        sub_context.borrow_mut().pop_token_from_stack();

        match result {
          Ok(success) => match success {
            MatcherSuccess::Token(token) => {
              if token.borrow().should_discard() {
                continue;
              }

              match self.on_first_match {
                MatchAction::Stop => {
                  if sub_context.borrow().debug_mode_level() > 1 {
                    println!(
                      "{{{}/Finalizing}}: First match success (Token)",
                      program_token.borrow().get_name(),
                    );
                  }

                  return Ok(MatcherSuccess::Token(token.clone()));
                }
                _ => {}
              }

              let is_consuming = pattern.borrow().is_consuming();

              handle_token(
                self,
                &program_token,
                &sub_context,
                &mut children,
                &mut captured_range,
                &mut matched_range,
                &token,
                is_consuming,
                is_consuming,
              );
            }
            MatcherSuccess::ProxyChildren(token) => {
              if token.borrow().should_discard() {
                continue;
              }

              match self.on_first_match {
                MatchAction::Stop => {
                  if sub_context.borrow().debug_mode_level() > 1 {
                    println!(
                      "{{{}/Finalizing}}: First match success (ProxyChildren)",
                      program_token.borrow().get_name(),
                    );
                  }

                  return Ok(MatcherSuccess::ProxyChildren(token.clone()));
                }
                _ => {}
              }

              let is_consuming = pattern.borrow().is_consuming();

              handle_extract_token(
                self,
                &program_token,
                &sub_context,
                &mut children,
                &mut captured_range,
                &mut matched_range,
                &token,
                is_consuming,
              );
            }
            MatcherSuccess::Skip(amount) => {
              if amount > 0 {
                match self.on_first_match {
                  MatchAction::Stop => {
                    if sub_context.borrow().debug_mode_level() > 1 {
                      println!(
                        "{{{}/Finalizing}}: First match success (Skip)",
                        program_token.borrow().get_name(),
                      );
                    }

                    return Ok(MatcherSuccess::Skip(amount));
                  }
                  _ => {}
                }
              }

              if pattern.borrow().is_consuming() {
                handle_skip(
                  self,
                  &sub_context,
                  &mut captured_range,
                  &mut matched_range,
                  start_offset,
                  amount,
                );
              }

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
              if sub_context.debug_mode_level() > 1 {
                print!("{{{}/Failure}} ", program_token.borrow().get_name());
              }

              println!(
                "`{}` Failure! -->|{}|--> @[{}-{}]",
                self.get_name(),
                sub_context.debug_range(10),
                sub_context.offset.start,
                sub_context.offset.end
              );
            }

            match failure {
              MatcherFailure::Fail => match self.on_first_match {
                MatchAction::Stop => {
                  continue;
                }
                _ => {
                  if is_loop {
                    if sub_context.debug_mode_level() > 1 {
                      println!(
                        "{{{}/Finalizing}}: Failure",
                        program_token.borrow().get_name()
                      );
                    }

                    return finalize_program_token(
                      program_token,
                      children,
                      captured_range,
                      matched_range,
                      &self.iterate_range,
                      loop_count,
                      false,
                    );
                  }

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

      match iteration_result {
        Some(action) => match action {
          MatcherSuccess::Break((loop_name, data)) => {
            match self.on_first_match {
              MatchAction::Stop => {
                if sub_context.borrow().debug_mode_level() > 1 {
                  println!(
                    "{{{}/Finalizing}}: First match success (Break)",
                    program_token.borrow().get_name(),
                  );
                }

                return Ok(MatcherSuccess::Break((loop_name, data)));
              }
              _ => {}
            }

            match &*data {
              MatcherSuccess::Token(token) => {
                handle_token(
                  self,
                  &program_token,
                  &sub_context,
                  &mut children,
                  &mut captured_range,
                  &mut matched_range,
                  &token,
                  false,
                  true,
                );

                Box::new(MatcherSuccess::None)
              }
              MatcherSuccess::ProxyChildren(token) => {
                handle_extract_token(
                  self,
                  &program_token,
                  &sub_context,
                  &mut children,
                  &mut captured_range,
                  &mut matched_range,
                  &token,
                  true,
                );

                Box::new(MatcherSuccess::None)
              }
              MatcherSuccess::Skip(amount) => {
                handle_skip(
                  self,
                  &sub_context,
                  &mut captured_range,
                  &mut matched_range,
                  start_offset,
                  *amount,
                );

                Box::new(MatcherSuccess::None)
              }
              _ => data,
            };

            if is_loop && (loop_name == self.name || loop_name == "") {
              if sub_context.borrow().debug_mode_level() > 1 {
                println!(
                  "{{{}/Finalizing}}: Consuming Break `{}`",
                  program_token.borrow().get_name(),
                  loop_name
                );
              }

              // This is the loop that should break, so cease propagating the Break
              return finalize_program_token(
                program_token,
                children,
                captured_range,
                matched_range,
                &self.iterate_range,
                loop_count,
                false,
              );
            } else {
              if sub_context.borrow().debug_mode_level() > 1 {
                println!(
                  "{{{}/Finalizing}}: Proxying Break `{}`",
                  program_token.borrow().get_name(),
                  loop_name
                );
              }

              match finalize_program_token(
                program_token,
                children,
                captured_range,
                matched_range,
                &self.iterate_range,
                loop_count,
                false,
              ) {
                Ok(final_token) => {
                  return Ok(MatcherSuccess::Break((loop_name, Box::new(final_token))));
                }
                Err(error) => {
                  return Err(error);
                }
              }
            }
          }
          MatcherSuccess::Continue((loop_name, data)) => {
            match self.on_first_match {
              MatchAction::Stop => {
                if sub_context.borrow().debug_mode_level() > 1 {
                  println!(
                    "{{{}/Finalizing}}: First match success (Continue)",
                    program_token.borrow().get_name(),
                  );
                }

                return Ok(MatcherSuccess::Break((loop_name, data)));
              }
              _ => {}
            }

            match &*data {
              MatcherSuccess::Token(token) => {
                handle_token(
                  self,
                  &program_token,
                  &sub_context,
                  &mut children,
                  &mut captured_range,
                  &mut matched_range,
                  &token,
                  false,
                  true,
                );

                Box::new(MatcherSuccess::None)
              }
              MatcherSuccess::ProxyChildren(token) => {
                handle_extract_token(
                  self,
                  &program_token,
                  &sub_context,
                  &mut children,
                  &mut captured_range,
                  &mut matched_range,
                  &token,
                  true,
                );

                Box::new(MatcherSuccess::None)
              }
              MatcherSuccess::Skip(amount) => {
                handle_skip(
                  self,
                  &sub_context,
                  &mut captured_range,
                  &mut matched_range,
                  start_offset,
                  *amount,
                );

                Box::new(MatcherSuccess::None)
              }
              _ => data,
            };

            // This is not the correct Loop, or is a Program, so propagate Continue
            if is_loop && (loop_name == self.name || loop_name == "") {
            } else {
              if sub_context.borrow().debug_mode_level() > 1 {
                println!(
                  "{{{}/Finalizing}}: Proxying Continue `{}`",
                  program_token.borrow().get_name(),
                  loop_name
                );
              }

              return finalize_program_token(
                program_token,
                children,
                captured_range,
                matched_range,
                &self.iterate_range,
                loop_count,
                false,
              );
            }
          }
          MatcherSuccess::Stop => {
            break;
          }
          _ => unreachable!(),
        },
        None => {}
      }

      loop_count += 1;
    }

    if sub_context.borrow().debug_mode_level() > 1 {
      println!(
        "{{{}/Finalizing}}: Completed!",
        program_token.borrow().get_name(),
      );
    }

    finalize_program_token(
      program_token,
      children,
      captured_range,
      matched_range,
      &self.iterate_range,
      loop_count,
      true,
    )
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

fn finalize_program_token(
  program_token: TokenRef,
  children: Vec<TokenRef>,
  captured_range: SourceRange,
  matched_range: SourceRange,
  iterate_range: &Option<Range<usize>>,
  loop_count: usize,
  fail_on_range_mismatch: bool,
) -> Result<MatcherSuccess, MatcherFailure> {
  // On "Break" from loop, we skip
  // this part, as we don't want
  // to trigger a failure due to a range
  // mismatch on a "Break"

  if fail_on_range_mismatch {
    if let Some(range) = iterate_range {
      if range.start == 0 && loop_count == 0 && children.len() == 0 {
        return Ok(MatcherSuccess::Skip(0));
      }

      // If we matched less than we were supposed to, then fail
      if loop_count < range.start {
        return Err(MatcherFailure::Fail);
      }
    }
  }

  // Fail if nothing was collected
  if captured_range.start == usize::MAX || matched_range.start == usize::MAX {
    return Err(MatcherFailure::Fail);
  }

  if children.len() < 1 {
    return Err(MatcherFailure::Fail);
  }

  {
    let mut program_token = program_token.borrow_mut();
    program_token.set_children(children);
    program_token.set_captured_range(captured_range);
    program_token.set_matched_range(matched_range);
  }

  Ok(MatcherSuccess::Token(program_token))
}

fn add_token_to_children(
  program_token: &TokenRef,
  context: &ParserContextRef,
  children: &mut Vec<TokenRef>,
  captured_range: &mut SourceRange,
  matched_range: &mut SourceRange,
  token: &TokenRef,
  assert_moving_forward: bool,
  update_offsets: bool,
) {
  {
    if !token.borrow().flags_enabled(IS_ERROR) {
      let token = token.borrow();

      if assert_moving_forward && update_offsets {
        // Ensure that we are moving forward, and that the token doesn't have a zero width
        if token.get_matched_range().end == context.borrow().offset.start {
          if context.borrow().debug_mode_level() > 1 {
            print!("{{{}/AddChild}} ", program_token.borrow().get_name());
          }

          println!(
            "`{}` Panic! Not moving forward when attempting to add child `{}`...",
            program_token.borrow().get_name(),
            token.get_name()
          );

          assert!(token.get_matched_range().end != context.borrow().offset.start);
        }
      }

      if update_offsets {
        // captured_range is set to matched_range because the program
        // should always span the range of all child tokens
        contain_source_range(captured_range, &token.get_captured_range());
        contain_source_range(matched_range, &token.get_matched_range());
      }
    } else {
      let mut token = token.borrow_mut();
      let source_range = token.get_matched_range();

      if source_range.start == source_range.end {
        let mut new_source_range = source_range.clone();
        new_source_range.start = matched_range.start;

        token.set_captured_range(new_source_range);
        token.set_matched_range(new_source_range);
      }

      if update_offsets {
        // captured_range is set to matched_range because the program
        // should always span the range of all child tokens
        contain_source_range(captured_range, &token.get_captured_range());
        contain_source_range(matched_range, &token.get_matched_range());
      }
    }
  }

  if update_offsets {
    context
      .borrow_mut()
      .set_start(token.borrow().get_matched_range().end);
  }

  {
    let mut token = token.borrow_mut();
    token.set_parent(Some(program_token.clone()));
  }

  children.push(token.clone());
}

fn handle_token(
  program: &ProgramPattern,
  program_token: &TokenRef,
  context: &ParserContextRef,
  children: &mut Vec<TokenRef>,
  captured_range: &mut SourceRange,
  matched_range: &mut SourceRange,
  token: &TokenRef,
  assert_moving_forward: bool,
  update_offsets: bool,
) {
  if context.borrow().debug_mode_level() > 0 {
    let token = token.borrow();

    if context.borrow().debug_mode_level() > 1 {
      print!("{{{}/Token}} ", program.get_name());
    }

    println!(
      "`{}` Adding child `{}` @[{}-{}]",
      program.get_name(),
      token.get_name(),
      token.get_matched_range().start,
      token.get_matched_range().end
    );
  }

  let should_discard = token.borrow().should_discard();

  add_token_to_children(
    &program_token,
    &context,
    children,
    captured_range,
    matched_range,
    token,
    assert_moving_forward,
    update_offsets && !should_discard,
  );

  if context.borrow().is_debug_mode() {
    if context.borrow().debug_mode_level() > 1 {
      print!("{{{}/Token}} ", program.get_name());
    }

    println!(
      "`{}` Setting to offset to: {} -> {}",
      program.get_name(),
      context.borrow().offset.start,
      token.borrow().get_matched_range().end
    );
  }
}

fn handle_extract_token(
  program: &ProgramPattern,
  program_token: &TokenRef,
  context: &ParserContextRef,
  children: &mut Vec<TokenRef>,
  captured_range: &mut SourceRange,
  matched_range: &mut SourceRange,
  token: &TokenRef,
  update_offsets: bool,
) {
  let token = token.borrow();
  let target_children = token.get_children();
  let should_discard = token.should_discard();

  if context.borrow().is_debug_mode() {
    if context.borrow().debug_mode_level() > 1 {
      print!("{{{}/ProxyChildren}} ", program.get_name());
    }

    println!(
      "`{}` Setting to offset to: {} -> {}",
      program.get_name(),
      context.borrow().offset.start,
      token.get_matched_range().end
    );
  }

  if update_offsets && !should_discard {
    context
      .borrow_mut()
      .set_start(token.get_matched_range().end);

    contain_source_range(captured_range, &token.get_captured_range());
    contain_source_range(matched_range, &token.get_matched_range());
  }

  if context.borrow().debug_mode_level() > 0 {
    if context.borrow().debug_mode_level() > 1 {
      print!("{{{}/ProxyChildren}} ", program.get_name());
    }

    let count = target_children.len();
    println!(
      "`{}` Will be adding {} {}",
      program.get_name(),
      count,
      if count != 1 { "children" } else { "child" }
    );
  }

  for child in target_children {
    if child.borrow().should_discard() {
      continue;
    }

    if context.borrow().debug_mode_level() > 0 {
      let child = child.borrow();

      if context.borrow().debug_mode_level() > 1 {
        print!("{{{}/ProxyChildren}} ", program.get_name());
      }

      println!(
        "`{}` Adding child `{}` @[{}-{}]",
        program.get_name(),
        child.get_name(),
        child.get_matched_range().start,
        child.get_matched_range().end
      );
    }

    add_token_to_children(
      program_token,
      context,
      children,
      captured_range,
      matched_range,
      &child,
      false,
      update_offsets && !should_discard,
    );
  }
}

fn handle_skip(
  program: &ProgramPattern,
  context: &ParserContextRef,
  _: &mut SourceRange,
  matched_range: &mut SourceRange,
  start_offset: usize,
  offset: isize,
) {
  let new_offset = context.borrow().offset.start + offset as usize;

  if context.borrow().is_debug_mode() {
    if context.borrow().debug_mode_level() > 1 {
      print!("{{{}/Skip}} ", program.get_name());
    }

    println!(
      "`{}` Skipping: {} + {} -> {}",
      program.get_name(),
      context.borrow().offset.start,
      offset,
      new_offset
    );
  }

  context.borrow_mut().set_start(new_offset);

  let range = SourceRange::new(start_offset, new_offset);

  contain_source_range(matched_range, &range);
}

impl Matcher for ProgramPattern {
  fn exec(
    &self,
    this_matcher: MatcherRef,
    context: ParserContextRef,
    scope: ScopeContextRef,
  ) -> Result<MatcherSuccess, MatcherFailure> {
    self.before_exec(this_matcher.clone(), context.clone(), scope.clone());
    let result = self._exec(context.clone(), scope.clone());
    self.after_exec(this_matcher.clone(), context.clone(), scope.clone());

    result
  }

  fn has_custom_name(&self) -> bool {
    self.custom_name
  }

  fn get_name(&self) -> &str {
    self.name.as_str()
  }

  fn set_name(&mut self, name: &str) {
    self.name = name.to_string();
    self.custom_name = true;
  }

  fn set_child(&mut self, index: usize, matcher: MatcherRef) {
    if index >= self.patterns.len() {
      panic!("Attempt to set child at an index that is out of bounds");
    }

    self.patterns[index] = matcher;
  }

  fn get_children(&self) -> Option<Vec<MatcherRef>> {
    Some(self.patterns.clone())
  }

  fn add_pattern(&mut self, pattern: MatcherRef) {
    self.patterns.push(pattern);
  }

  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

#[macro_export]
macro_rules! Program {
  ($name:literal; $($args:expr),+ $(,)?) => {
    {
      let program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Continue);

      {
        let mut pm = program.borrow_mut();
        pm.set_name($name);

        $(
          pm.add_pattern($args);
        )*
      }

      program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Continue);

      {
        let mut pm = program.borrow_mut();

        $(
          pm.add_pattern($args);
        )*
      }

      program
    }
  };

  () => {
    {
      $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Continue)
    }
  };
}

#[macro_export]
macro_rules! Switch {
  ($name:literal; $($args:expr),+ $(,)?) => {
    {
      let program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Stop);

      {
        let mut pm = program.borrow_mut();
        pm.set_name($name);

        $(
          pm.add_pattern($args);
        )*
      }

      program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let program = $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Stop);

      {
        let mut pm = program.borrow_mut();

        $(
          pm.add_pattern($args);
        )*
      }

      program
    }
  };

  () => {
    {
      $crate::matchers::program::ProgramPattern::new_blank_program($crate::matchers::program::MatchAction::Stop)
    }
  };
}

#[macro_export]
macro_rules! Loop {
  ($range:expr; $name:literal; $($args:expr),+ $(,)?) => {
    {
      let loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop($range);

      {
        let mut lm = loop_program.borrow_mut();
        lm.set_name($name);

        $(
          lm.add_pattern($args);
        )*
      }

      loop_program
    }
  };

  ($name:literal; $($args:expr),+ $(,)?) => {
    {
      let loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop(0..);

      {
        let mut lm = loop_program.borrow_mut();
        lm.set_name($name);

        $(
          lm.add_pattern($args);
        )*
      }

      loop_program
    }
  };

  ($range:expr; $($args:expr),+ $(,)?) => {
    {
      let loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop($range);

      {
        let mut lm = loop_program.borrow_mut();

        $(
          lm.add_pattern($args);
        )*
      }

      loop_program
    }
  };

  ($($args:expr),+ $(,)?) => {
    {
      let loop_program = $crate::matchers::program::ProgramPattern::new_blank_loop(0..);

      {
        let mut lm = loop_program.borrow_mut();

        $(
          lm.add_pattern($args);
        )*
      }

      loop_program
    }
  };

  () => {
    {
      $crate::matchers::program::ProgramPattern::new_blank_loop(0..)
    }
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::MatcherFailure, parser::Parser, parser_context::ParserContext,
    source_range::SourceRange, Break, Equals, Matches, Optional,
  };

  #[test]
  fn it_matches_against_a_simple_program() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Equals!("Testing"), Equals!(" "), Matches!(r"\d+"));

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Program");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), &parser.borrow().source);
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
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_matches_against_a_simple_switch() {
    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Switch!(Equals!(" "), Matches!(r"\d+"), Equals!("Testing"));

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Equals");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 7));
      assert_eq!(token.get_value(), "Testing");
      assert_eq!(token.get_matched_value(), "Testing");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_matches_against_a_loop() {
    let parser = Parser::new("A B C D E F ");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Loop!(Matches!(r"\w"), Equals!(" "));

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), &parser.borrow().source);
      assert_eq!(token.get_matched_value(), &parser.borrow().source);

      assert_eq!(token.get_children().len(), 12);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        assert_eq!(
          token.get_children()[index * 2].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          token.get_children()[index * 2]
            .borrow()
            .get_captured_value(),
          parts[index]
        );
        assert_eq!(
          token.get_children()[index * 2 + 1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(
          token.get_children()[index * 2 + 1]
            .borrow()
            .get_captured_value(),
          " "
        );
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

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 12));
      assert_eq!(token.get_value(), &parser.borrow().source);
      assert_eq!(token.get_matched_value(), &parser.borrow().source);

      assert_eq!(token.get_children().len(), 6);

      let parts = vec!["A", "B", "C", "D", "E", "F"];

      for index in 0..parts.len() {
        let program_token = &token.get_children()[index];

        assert_eq!(
          program_token.borrow().get_children()[0].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          program_token.borrow().get_children()[0]
            .borrow()
            .get_captured_value(),
          parts[index]
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(
          program_token.borrow().get_children()[1]
            .borrow()
            .get_captured_value(),
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

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      let token = token.borrow();
      assert_eq!(token.get_name(), "Loop");
      assert_eq!(*token.get_captured_range(), SourceRange::new(0, 11));
      assert_eq!(token.get_value(), "A B C break");
      assert_eq!(token.get_matched_value(), "A B C break");

      assert_eq!(token.get_children().len(), 4);

      let parts = vec!["A", "B", "C"];

      for index in 0..parts.len() {
        let program_token = &token.get_children()[index];

        assert_eq!(
          program_token.borrow().get_children()[0].borrow().get_name(),
          "Matches"
        );
        assert_eq!(
          program_token.borrow().get_children()[0]
            .borrow()
            .get_captured_value(),
          parts[index]
        );
        assert_eq!(
          program_token.borrow().get_children()[1].borrow().get_name(),
          "Equals"
        );
        assert_eq!(
          program_token.borrow().get_children()[1]
            .borrow()
            .get_captured_value(),
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
        program_token.borrow().get_children()[0]
          .borrow()
          .get_captured_value(),
        "break"
      );
      assert_eq!(
        program_token.borrow().get_children()[0]
          .borrow()
          .get_matched_value(),
        "break"
      );
    } else {
      unreachable!("Test failed!");
    };
  }
}
