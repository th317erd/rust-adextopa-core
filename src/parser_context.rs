use super::source_range::SourceRange;
use crate::{
  matcher::{MatcherFailure, MatcherRef, MatcherSuccess},
  parser::ParserRef,
  scope::VariableType,
  scope_context::{ScopeContext, ScopeContextRef},
  token::{TokenRef, IS_ERROR},
};
use regex::Regex;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type ParserContextRef = Rc<RefCell<ParserContext>>;

lazy_static::lazy_static! {
  static ref NEWLINES: regex::Regex = regex::Regex::new(r"(\r\n|\n|\r)").expect("Could not compile needed Regex for `parser_context`");
}

#[derive(Clone)]
pub struct ParserContext {
  pub(crate) debug_mode: usize,
  pub(crate) scope: ScopeContextRef,
  pub(crate) token_stack: Vec<TokenRef>,
  pub offset: SourceRange,
  pub parser: ParserRef,
  pub name: String,
}

impl ParserContext {
  pub fn new(parser: &ParserRef, name: &str) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      scope: ScopeContext::new(),
      token_stack: vec![],
      offset: SourceRange::new(0, parser.borrow().source.len()),
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn new_with_offset(parser: &ParserRef, offset: SourceRange, name: &str) -> ParserContextRef {
    std::rc::Rc::new(std::cell::RefCell::new(ParserContext {
      scope: ScopeContext::new(),
      token_stack: vec![],
      offset,
      parser: parser.clone(),
      debug_mode: 0,
      name: name.to_string(),
    }))
  }

  pub fn clone_with_name(&self, name: &str) -> ParserContextRef {
    let mut c = self.clone();
    c.name = name.to_string();
    std::rc::Rc::new(std::cell::RefCell::new(c))
  }

  pub fn is_debug_mode(&self) -> bool {
    self.debug_mode > 0
  }

  pub fn debug_mode_level(&self) -> usize {
    self.debug_mode
  }

  pub fn set_debug_mode(&mut self, value: usize) {
    self.debug_mode = value;
  }

  pub fn set_start(&mut self, start: usize) {
    self.offset.start = start;
  }

  pub fn set_end(&mut self, end: usize) {
    self.offset.end = end;
  }

  pub fn set_offset(&mut self, range: SourceRange) {
    self.offset = range;
  }

  pub fn get_scope_variable(&self, name: &str) -> Option<VariableType> {
    self.scope.borrow().get(name)
  }

  pub fn set_scope_variable(&mut self, name: &str, value: VariableType) -> Option<VariableType> {
    self.scope.borrow_mut().set(name, value)
  }

  pub fn matches_str(&self, pattern: &str) -> Option<SourceRange> {
    if pattern.len() == 0 {
      return None;
    }

    let chunk = &self.parser.borrow().source[self.offset.start..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(self.offset.clone_with_len(pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_str_at_offset(&self, pattern: &str, offset: usize) -> Option<SourceRange> {
    if offset >= self.offset.end || pattern.len() == 0 {
      return None;
    }

    let chunk = &self.parser.borrow().source[offset..self.offset.end];

    if chunk.starts_with(pattern) {
      Some(SourceRange::new(offset, offset + pattern.len()))
    } else {
      None
    }
  }

  pub fn matches_regexp(&self, pattern: &Regex) -> Option<SourceRange> {
    let chunk = &self.parser.borrow().source[self.offset.start..self.offset.end];

    match pattern.find(chunk) {
      Some(m) => {
        let start = m.start() + self.offset.start;
        let end = m.end() + self.offset.start;

        if start != self.offset.start {
          None
        } else {
          Some(SourceRange::new(start, end))
        }
      }
      None => None,
    }
  }

  pub fn debug_range(&self, max_len: usize) -> String {
    let parser = self.parser.borrow();
    let mut end_offset = self.offset.start + max_len;

    if end_offset > self.offset.end {
      end_offset = self.offset.end;
    }

    parser.source[self.offset.start..end_offset].to_string()
  }

  // pub fn capture_matcher_references(&self, matcher: MatcherRef) {
  //   let m = matcher.borrow();

  //   if m.has_custom_name() {
  //     let name = m.get_name();

  //     if self.debug_mode > 1 {
  //       println!("Registering matcher `{}`", name);
  //     }

  //     self
  //       .scope
  //       .borrow_mut()
  //       .set(name, VariableType::Matcher(matcher.clone()));
  //   }

  //   match m.get_children() {
  //     Some(children) => {
  //       for child in children {
  //         self.capture_matcher_references(child.clone());
  //       }
  //     }
  //     None => {}
  //   }
  // }

  pub fn push_token_to_stack(&mut self, token: TokenRef) {
    self.token_stack.push(token.clone())
  }

  pub fn pop_token_from_stack(&mut self) {
    self.token_stack.pop();
  }

  pub fn get_top_token_from_stack(&self) -> Option<TokenRef> {
    if self.token_stack.len() == 0 {
      return None;
    }

    Some(self.token_stack[self.token_stack.len() - 1].clone())
  }

  pub fn get_lines(&self, range: &SourceRange) -> (usize, usize) {
    let parser = self.parser.borrow();
    let source = parser.source.as_str();

    let first_line = NEWLINES.find_iter(&source[0..range.start]).count() + 1;
    let last_line = NEWLINES.find_iter(&source[0..range.end]).count() + 1;

    return (first_line, last_line);
  }

  pub fn get_columns(&self, range: &SourceRange) -> (usize, usize) {
    let parser = self.parser.borrow();
    let source = parser.source.as_str();
    let mut last_newline = 0;
    let mut first_column = 0;
    let mut last_column = 0;

    for item in NEWLINES.find_iter(&source[0..range.end]) {
      let end_position = item.end();

      if first_column == 0 && end_position > range.start {
        first_column = (range.start - last_newline) + 1;
      }

      if end_position > range.end {
        last_column = (range.end - last_newline) + 1;
        break;
      }

      last_newline = end_position;
    }

    if first_column == 0 {
      if last_newline != 0 {
        first_column = (range.start - last_newline) + 1;
      } else {
        first_column = range.start + 1;
      }
    }

    if last_column == 0 {
      if last_newline != 0 {
        last_column = (range.end - last_newline) + 1;
      } else {
        last_column = range.end + 1;
      }
    }

    if first_column > last_column {
      last_column = first_column;
    }

    return (first_column, last_column);
  }

  pub fn get_lines_and_columns(&self, range: &SourceRange) -> ((usize, usize), (usize, usize)) {
    return (self.get_lines(range), self.get_columns(range));
  }

  pub fn get_error_as_string(&self, message: &str, range: &SourceRange) -> String {
    let (lines, columns) = self.get_lines_and_columns(range);
    let parser = self.parser.borrow();
    let filename = &parser.filename;

    let line_str = if lines.0 == lines.1 {
      format!("{}", lines.0)
    } else {
      format!("{}-{}", lines.0, lines.1)
    };

    let column_str = if columns.0 == columns.1 {
      format!("{}", columns.0)
    } else {
      format!("{}-{}", columns.0, columns.1)
    };

    format!(
      "Error: {}@[{}:{}]: {}",
      filename, line_str, column_str, message
    )
  }

  pub fn display_error(&self, message: &str, _: &SourceRange) {
    println!("{}", message);
  }

  pub fn register_matchers(&self, matchers: Vec<MatcherRef>) {
    for matcher in matchers {
      let _matcher = matcher.borrow();
      let name = _matcher.get_name();

      self
        .scope
        .borrow_mut()
        .set(name, VariableType::Matcher(matcher.clone()));
    }
  }

  pub fn register_matchers_with_names(&self, matchers: HashMap<&str, MatcherRef>) {
    for (name, matcher) in matchers {
      let _matcher = matcher.borrow();

      self
        .scope
        .borrow_mut()
        .set(name, VariableType::Matcher(matcher.clone()));
    }
  }

  pub fn register_matcher_with_name(&self, name: &str, matcher: MatcherRef) {
    self
      .scope
      .borrow_mut()
      .set(name, VariableType::Matcher(matcher.clone()));
  }

  pub fn register_matcher(&self, matcher: MatcherRef) {
    let _matcher = matcher.borrow();
    let name = _matcher.get_name();
    self.register_matcher_with_name(name, matcher.clone());
  }

  pub fn get_registered_matcher(&self, name: &str) -> Option<MatcherRef> {
    match self.scope.borrow().get(name) {
      Some(VariableType::Matcher(matcher)) => Some(matcher.clone()),
      _ => None,
    }
  }

  fn collect_errors(root_token: TokenRef, token: TokenRef, is_root: bool) {
    if is_root {
      // We need to make a copy of this vector so that we can drop
      // the _token reference... we only want to do this for the root
      // node, as it doesn't matter for other nodes.
      let _token = token.borrow();
      let children = _token.get_children().clone();

      drop(_token);

      for child in children {
        Self::collect_errors(root_token.clone(), child.clone(), false);
      }
    } else {
      for child in token.borrow().get_children() {
        Self::collect_errors(root_token.clone(), child.clone(), false);
      }
    }

    if token.borrow().get_children().len() > 0 {
      if is_root == false {
        let mut current_node = token.borrow_mut();
        let children = current_node.get_children_mut();

        // Filter out errors
        children.retain(|token| {
          let is_error = token.borrow().flags_enabled(IS_ERROR);

          if is_error {
            root_token
              .borrow_mut()
              .get_children_mut()
              .push(token.clone());

            false
          } else {
            true
          }
        });
      }
    }
  }

  pub fn tokenize(
    context: ParserContextRef,
    matcher: MatcherRef,
  ) -> Result<TokenRef, MatcherFailure> {
    //context.borrow().capture_matcher_references(matcher.clone());

    let scope = context.borrow().scope.clone();

    match matcher
      .borrow()
      .exec(matcher.clone(), context.clone(), scope)
    {
      Ok(success) => match success {
        MatcherSuccess::Token(ref token) => {
          Self::collect_errors(token.clone(), token.clone(), true);
          return Ok(token.clone());
        }
        MatcherSuccess::ExtractChildren(token) => {
          Self::collect_errors(token.clone(), token.clone(), true);
          return Ok(token.clone());
        }
        MatcherSuccess::Break((_, token)) => match &*token {
          MatcherSuccess::Token(ref token) => {
            Self::collect_errors(token.clone(), token.clone(), true);
            return Ok(token.clone());
          }
          _ => Err(MatcherFailure::Fail),
        },
        MatcherSuccess::Continue((_, token)) => match &*token {
          MatcherSuccess::Token(ref token) => {
            Self::collect_errors(token.clone(), token.clone(), true);
            return Ok(token.clone());
          }
          _ => Err(MatcherFailure::Fail),
        },
        _ => Err(MatcherFailure::Fail),
      },
      Err(error) => Err(error),
    }
  }
}

#[cfg(test)]
mod test {
  use super::ParserContext;
  use crate::{parser::Parser, source_range::SourceRange};

  #[test]
  fn get_line_works() {
    let parser = Parser::new("Test 1\nTest 2\r\nTest 3\rTest 4");
    let parser_context = ParserContext::new(&parser, "Test");

    assert_eq!(
      parser_context.borrow().get_lines(&SourceRange::new(4, 6)),
      (1, 1)
    );
    assert_eq!(
      parser_context.borrow().get_lines(&SourceRange::new(8, 9)),
      (2, 2)
    );
    assert_eq!(
      parser_context.borrow().get_lines(&SourceRange::new(14, 15)),
      (3, 3)
    );
    assert_eq!(
      parser_context.borrow().get_lines(&SourceRange::new(23, 24)),
      (4, 4)
    );
  }

  #[test]
  fn get_column_works() {
    let parser = Parser::new("Test 1\nTest 2\r\nTest 3\rTest 4");
    let parser_context = ParserContext::new(&parser, "Test");

    assert_eq!(
      parser_context.borrow().get_columns(&SourceRange::new(0, 2)),
      (1, 3)
    );
    assert_eq!(
      parser_context.borrow().get_columns(&SourceRange::new(8, 9)),
      (2, 3)
    );
    assert_eq!(
      parser_context
        .borrow()
        .get_columns(&SourceRange::new(13, 15)),
      (7, 7)
    );
  }

  #[test]
  fn get_line_and_column_works() {
    let parser = Parser::new("Test 1\nTest 2\r\nTest 3\rTest 4");
    let parser_context = ParserContext::new(&parser, "Test");

    assert_eq!(
      parser_context
        .borrow()
        .get_lines_and_columns(&SourceRange::new(0, 2)),
      ((1, 1), (1, 3))
    );
    assert_eq!(
      parser_context
        .borrow()
        .get_lines_and_columns(&SourceRange::new(8, 9)),
      ((2, 2), (2, 3))
    );
    assert_eq!(
      parser_context
        .borrow()
        .get_lines_and_columns(&SourceRange::new(15, 17)),
      ((3, 3), (1, 3))
    );
    assert_eq!(
      parser_context
        .borrow()
        .get_lines_and_columns(&SourceRange::new(0, 19)),
      ((1, 3), (1, 5))
    );
  }
}
