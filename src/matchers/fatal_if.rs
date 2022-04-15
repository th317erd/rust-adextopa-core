#[macro_export]
macro_rules! FatalIf {
  ($matcher:expr, $message:expr) => {
    $crate::Map!(
      $matcher,
      |token, context, _| {
        // If the match succeeded, then throw an error
        let _token = token.borrow();
        let range = _token.get_matched_range();

        Err($crate::matcher::MatcherFailure::Error(
          $crate::parse_error::ParseError::new_with_range(
            &context.borrow().get_error_as_string($message, &range),
            range.clone(),
          ),
        ))
      },
      |failure, _, __| {
        // If the matched failed... then we are good
        // ... unless it was an error, in which case
        // continue proxying error up-stream
        match failure {
          $crate::matcher::MatcherFailure::Fail => Ok($crate::matcher::MatcherSuccess::Skip(0)),
          $crate::matcher::MatcherFailure::Error(error) => {
            Err($crate::matcher::MatcherFailure::Error(error))
          }
        }
      }
    )
  };
}
