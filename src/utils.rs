use crate::{source_range::SourceRange, token::TokenRef};

pub(crate) fn containing_value_range<'a>(tokens: &Vec<TokenRef>) -> SourceRange {
  let mut source_range = SourceRange::new(usize::MAX, 0);

  for t in tokens {
    let token = t.borrow();
    let range = token.get_value_range();

    if range.start < source_range.start {
      source_range.start = range.start;
    }

    if range.end > source_range.end {
      source_range.end = range.end;
    }
  }

  source_range
}

pub(crate) fn containing_raw_range<'a>(tokens: &Vec<TokenRef>) -> SourceRange {
  let mut source_range = SourceRange::new(usize::MAX, 0);

  for t in tokens {
    let token = t.borrow();
    let range = token.get_raw_range();

    if range.start < source_range.start {
      source_range.start = range.start;
    }

    if range.end > source_range.end {
      source_range.end = range.end;
    }
  }

  source_range
}
