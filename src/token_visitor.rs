use std::{cell::RefCell, rc::Rc};

use crate::token::TokenRef;

pub struct TokenVisitor<'a> {
  hooks: Vec<(
    &'a str,
    Rc<RefCell<Box<dyn FnMut(TokenRef) -> Result<(), String> + 'a>>>,
  )>,
}

impl<'a> TokenVisitor<'a> {
  pub fn new() -> Self {
    Self { hooks: Vec::new() }
  }

  pub fn add_visitor(
    &mut self,
    name: &'a str,
    func: Box<dyn FnMut(TokenRef) -> Result<(), String> + 'a>,
  ) {
    self.hooks.push((name, Rc::new(RefCell::new(func))));
  }

  fn find_hook(
    &mut self,
    name: &str,
  ) -> Option<Rc<RefCell<Box<dyn FnMut(TokenRef) -> Result<(), String> + 'a>>>> {
    for hook in &mut self.hooks {
      if hook.0 == name {
        return Some(hook.1.clone());
      }
    }

    None
  }

  fn _visit(
    &mut self,
    default_hook: Option<Rc<RefCell<Box<dyn FnMut(TokenRef) -> Result<(), String> + 'a>>>>,
    token: TokenRef,
  ) -> Result<(), String> {
    let _token = token.borrow();

    if let Some(ref hook) = self.find_hook(_token.get_name()) {
      match (hook.clone().borrow_mut())(token.clone()) {
        Err(err) => return Err(err),
        Ok(_) => {}
      }
    } else if default_hook.is_some() {
      let dh = default_hook.as_ref().unwrap();
      match (dh.clone().borrow_mut())(token.clone()) {
        Err(err) => return Err(err),
        Ok(_) => {}
      }
    }

    for child in _token.get_children() {
      let _child = child.borrow();

      let result = match default_hook {
        Some(ref default_hook) => self._visit(Some(default_hook.clone()), child.clone()),
        None => self._visit(None, child.clone()),
      };

      if let Err(error) = result {
        return Err(error);
      };
    }

    Ok(())
  }

  pub fn visit(&mut self, token: TokenRef) -> Result<(), String> {
    let default_hook = self.find_hook("*");
    self._visit(default_hook, token)
  }
}

#[macro_export]
macro_rules! Visit {
  ($token:expr, $context:expr, $($key:expr => $handler:expr),+ $(,)?) => {
    {
      let mut visitors = $crate::token_visitor::TokenVisitor::new();

      $(
        visitors.add_visitor($key, Box::new($handler));
      )*

      visitors.visit($token.clone())
    }
  };
}

#[cfg(test)]
mod test {
  use crate::{parser::Parser, parser_context::ParserContext, Equals, Loop, Matches, Program};

  #[test]
  fn it_works() {
    let parser = Parser::new("Testing 1234 one two three");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(
      Program!(Equals!("Testing"), Equals!(" "), Matches!(r"\d+")),
      Loop!(Matches!(r"\s"), Matches!(r"\w+"))
    );

    if let Ok(token) = ParserContext::tokenize(parser_context, matcher) {
      // Visit everyone
      let context = std::cell::RefCell::new(Vec::<String>::new());
      let mut x: i32 = 0;

      let result = Visit!(token, context,
        "Equals" => |token| {
          let mut context = context.borrow_mut();
          let token = token.borrow();
          context.push(format!("Equals = {}", token.get_value()));
          x = 12;
          Ok(())
        },
        "*" => |token| {
          let mut context = context.borrow_mut();
          let token = token.borrow();
          context.push(format!("{}: {}", token.get_name(), token.get_value()));
          Ok(())
        }
      );

      if let Ok(_) = result {
        assert_eq!(x, 12);
        assert_eq!(
          *context.borrow(),
          vec![
            "Program: Testing 1234 one two three",
            "Program: Testing 1234",
            "Equals = Testing",
            "Equals =  ",
            "Matches: 1234",
            "Loop:  one two three",
            "Matches:  ",
            "Matches: one",
            "Matches:  ",
            "Matches: two",
            "Matches:  ",
            "Matches: three"
          ]
        );
      } else {
        unreachable!("Test failed!");
      };
    } else {
      unreachable!("Test failed!");
    }
  }
}
