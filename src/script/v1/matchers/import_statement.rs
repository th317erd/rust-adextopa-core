#[macro_export]
macro_rules! ScriptImportStatement {
  () => {
    $crate::Program!("ImportStatement";
      $crate::Discard!($crate::Equals!("import")),
      $crate::ScriptWS0!(?),
      $crate::Program!("ImportIdentifiers";
        $crate::Discard!($crate::Equals!("{")),
        $crate::Flatten!(
          $crate::Loop!("IdentifiersLoop";
            $crate::ScriptWSN0!(?),
            $crate::Switch!(
              $crate::Discard!(
                $crate::Program!(
                  $crate::Equals!("}"),
                  $crate::Break!("IdentifiersLoop"),
                )
              ),
              $crate::Program!("ImportIdentifier";
                $crate::ScriptIdentifier!("ImportName"),
                $crate::Optional!(
                  $crate::Flatten!(
                    $crate::Program!(
                      $crate::Discard!($crate::Matches!(r"\s+as\s+")),
                      $crate::ScriptIdentifier!("ImportAsName"),
                    )
                  )
                ),
                $crate::Discard!(
                  $crate::Optional!($crate::Equals!(","))
                )
              ),
            )
          )
        ),
      ),
      $crate::ScriptWS0!(?),
      $crate::Discard!($crate::Equals!("from")),
      $crate::ScriptWS0!(?),
      $crate::ScriptString!("Path"),
    )
  };
}

#[cfg(test)]
mod tests {
  use crate::{
    matcher::{MatcherFailure},
    parser::Parser,
    parser_context::ParserContext,
    source_range::SourceRange,
  };

  #[test]
  fn it_works1() {
    let parser = Parser::new("import { _ as derp, Stuff as Things, Wow } from '../test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    let result = ParserContext::tokenize(parser_context, matcher);

    if let Ok(token) = result {
      let token = token.borrow();
      assert_eq!(token.get_name(), "ImportStatement");
      assert_eq!(*token.get_captured_range(), SourceRange::new(9, 56));
      assert_eq!(*token.get_matched_range(), SourceRange::new(0, 57));
      assert_eq!(
        token.get_value(),
        "_ as derp, Stuff as Things, Wow } from '../test"
      );
      assert_eq!(
        token.get_matched_value(),
        "import { _ as derp, Stuff as Things, Wow } from '../test'"
      );
      assert_eq!(token.get_children().len(), 2);

      let first = token.get_children()[0].borrow();
      assert_eq!(first.get_name(), "ImportIdentifiers");
      assert_eq!(*first.get_captured_range(), SourceRange::new(9, 40));
      assert_eq!(*first.get_matched_range(), SourceRange::new(7, 42));
      assert_eq!(first.get_value(), "_ as derp, Stuff as Things, Wow");
      assert_eq!(
        first.get_matched_value(),
        "{ _ as derp, Stuff as Things, Wow }"
      );
      assert_eq!(first.get_children().len(), 3);

      let ident_first = first.get_children()[0].borrow();
      assert_eq!(ident_first.get_name(), "ImportIdentifier");
      assert_eq!(*ident_first.get_captured_range(), SourceRange::new(9, 18));
      assert_eq!(*ident_first.get_matched_range(), SourceRange::new(9, 19));
      assert_eq!(ident_first.get_value(), "_ as derp");
      assert_eq!(ident_first.get_matched_value(), "_ as derp,");
      assert_eq!(ident_first.get_children().len(), 2);

      let ident_second = first.get_children()[1].borrow();
      assert_eq!(ident_second.get_name(), "ImportIdentifier");
      assert_eq!(*ident_second.get_captured_range(), SourceRange::new(20, 35));
      assert_eq!(*ident_second.get_matched_range(), SourceRange::new(20, 36));
      assert_eq!(ident_second.get_value(), "Stuff as Things");
      assert_eq!(ident_second.get_matched_value(), "Stuff as Things,");
      assert_eq!(ident_second.get_children().len(), 2);

      let ident_third = first.get_children()[2].borrow();
      assert_eq!(ident_third.get_name(), "ImportIdentifier");
      assert_eq!(*ident_third.get_captured_range(), SourceRange::new(37, 40));
      assert_eq!(*ident_third.get_matched_range(), SourceRange::new(37, 40));
      assert_eq!(ident_third.get_value(), "Wow");
      assert_eq!(ident_third.get_matched_value(), "Wow");
      assert_eq!(ident_third.get_children().len(), 1);

      let second = token.get_children()[1].borrow();
      assert_eq!(second.get_name(), "Path");
      assert_eq!(*second.get_captured_range(), SourceRange::new(49, 56));
      assert_eq!(*second.get_matched_range(), SourceRange::new(48, 57));
      assert_eq!(second.get_value(), "../test");
      assert_eq!(second.get_matched_value(), "'../test'");
    } else {
      unreachable!("Test failed!");
    };
  }

  #[test]
  fn it_fails1() {
    let parser = Parser::new("Testing 'derp'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }

  #[test]
  fn it_fails2() {
    let parser = Parser::new("import\n'test'");
    let parser_context = ParserContext::new(&parser, "Test");
    let matcher = ScriptImportStatement!();

    assert_eq!(
      Err(MatcherFailure::Fail),
      ParserContext::tokenize(parser_context, matcher)
    );
  }
}
