use crate::token::TokenRef;

struct TokenVisitor<'a, T> {
  hooks: Vec<(
    &'a str,
    Box<dyn Fn(TokenRef, &mut T) -> Result<&mut T, String>>,
  )>,
}

impl<'a, T> TokenVisitor<'a, T> {
  pub fn new() -> Self {
    Self { hooks: Vec::new() }
  }

  pub fn add_visitor(
    &mut self,
    name: &'a str,
    func: Box<dyn Fn(TokenRef, &mut T) -> Result<&mut T, String>>,
  ) {
    self.hooks.push((name, func));
  }

  fn find_hook(&self, name: &str) -> Option<&dyn Fn(TokenRef, &mut T) -> Result<&mut T, String>> {
    for hook in &self.hooks {
      if hook.0 == name {
        return Some(&hook.1);
      }
    }

    None
  }

  fn _visit<'b>(
    &self,
    default_hook: Option<&dyn Fn(TokenRef, &mut T) -> Result<&mut T, String>>,
    token: TokenRef,
    context: &'b mut T,
  ) -> Result<&'b mut T, String> {
    let _token = token.borrow();

    if let Some(hook) = self.find_hook(_token.get_name()) {
      match (*hook)(token.clone(), context) {
        Err(err) => return Err(err),
        Ok(_) => {}
      }
    } else if default_hook.is_some() {
      match (*default_hook.unwrap())(token.clone(), context) {
        Err(err) => return Err(err),
        Ok(_) => {}
      }
    }

    for child in _token.get_children() {
      let _child = child.borrow();

      if let Some(hook) = self.find_hook(_child.get_name()) {
        match (*hook)(child.clone(), context) {
          Err(err) => return Err(err),
          Ok(_) => {}
        }
      } else if default_hook.is_some() {
        match (*default_hook.unwrap())(child.clone(), context) {
          Err(err) => return Err(err),
          Ok(_) => {}
        }
      }
    }

    Ok(context)
  }

  pub fn visit<'b>(&'a self, token: TokenRef, context: &'b mut T) -> Result<&'b mut T, String> {
    if let Some(default_hook) = self.find_hook("*") {
      self._visit(Some(default_hook), token, context)
    } else {
      self._visit(None, token, context)
    }
  }
}

#[macro_export]
macro_rules! Visitors {
  ($type:ty; $($key:expr => $handler:expr),+ $(,)?) => {
    {
      let mut visitors = TokenVisitor::<$type>::new();

      $(
        visitors.add_visitor($key, Box::new($handler));
      )*

      visitors
    }
  };

  ($($key:expr => $handler:expr),+ $(,)?) => {
    {
      let mut visitors = TokenVisitor::new();

      $(
        visitors.add_visitor($key, Box::new($handler));
      )*

      visitors
    }
  };
}

#[cfg(test)]
mod test {
  use crate::{
    matcher::MatcherSuccess, parser::Parser, parser_context::ParserContext, token::TokenRef,
    Equals, Matches, Program,
  };

  use super::TokenVisitor;

  #[test]
  fn it_works() {
    let mut context = Vec::<String>::new();

    let visitors = Visitors!(Vec::<String>;
      "Equals" => |token, context| {
        let token = token.borrow();
        context.push(format!("Equals = {}", token.value()));
        Ok(context)
      },
      "*" => |token, context| {
        let token = token.borrow();
        context.push(format!("{}: {}", token.get_name(), token.value()));
        Ok(context)
      }
    );

    let parser = Parser::new("Testing 1234");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = Program!(Equals!("Testing"), Equals!(" "), Matches!(r"\d+"));

    if let Ok(MatcherSuccess::Token(token)) = ParserContext::tokenize(parser_context, matcher) {
      // Visit everyone
      let result = visitors.visit(token.clone(), &mut context);

      if let Ok(result) = result {
        assert_eq!(
          *result,
          vec![
            "Program: Testing 1234",
            "Equals = Testing",
            "Equals =  ",
            "Matches: 1234"
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
