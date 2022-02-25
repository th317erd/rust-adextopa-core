use crate::{source_range::SourceRange, token::TokenRef};

pub(crate) fn containing_value_range<'a>(tokens: &Vec<TokenRef<'a>>) -> SourceRange {
  let mut source_range = SourceRange::new(usize::MAX, 0);

  for t in tokens {
    let token = t.borrow();
    if token.value_range.start < source_range.start {
      source_range.start = token.value_range.start;
    }

    if token.value_range.end > source_range.end {
      source_range.end = token.value_range.end;
    }
  }

  source_range
}

pub(crate) fn containing_raw_range<'a>(tokens: &Vec<TokenRef<'a>>) -> SourceRange {
  let mut source_range = SourceRange::new(usize::MAX, 0);

  for t in tokens {
    let token = t.borrow();
    if token.raw_range.start < source_range.start {
      source_range.start = token.raw_range.start;
    }

    if token.raw_range.end > source_range.end {
      source_range.end = token.raw_range.end;
    }
  }

  source_range
}
