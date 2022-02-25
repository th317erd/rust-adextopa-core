use super::parser_context::ParserContext;
use super::token::TokenRef;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum MatcherSuccess<'a> {
  Token(TokenRef<'a>),
  Skip(isize),
  Break((&'a str, Box<MatcherSuccess<'a>>)),
  Continue((&'a str, Box<MatcherSuccess<'a>>)),
  None,
  Stop,
}

#[derive(Debug, PartialEq)]
pub enum MatcherFailure<'a> {
  Fail,
  Error(&'a str),
}

pub enum Pattern<'a> {
  String(&'a str),
  RegExp(Regex),
  Matcher(&'a dyn Matcher),
  Func(&'a dyn Fn(&'a ParserContext) -> Result<MatcherSuccess<'a>, MatcherFailure<'a>>),
}

pub trait Matcher {
  fn exec(&self, context: &ParserContext) -> Result<MatcherSuccess, MatcherFailure>;
  fn get_name(&self) -> &str;
}
