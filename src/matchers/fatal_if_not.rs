#[macro_export]
macro_rules! FatalIfNot {
  ($matcher:expr, $message:expr) => {
    $crate::Map!(
      $matcher,
      |token, context, _| {
        // If success, then don't fail
        // instead, pass a "Skip" command
        // upstream to succeed silently
        Ok($crate::matcher::MatcherSuccess::Skip(0))
      },
      |failure, context, __| {
        // If the matched failed then we failed
        // Proxy errors up-stream as usual
        match failure {
          $crate::matcher::MatcherFailure::Fail => {
            let range = context.borrow().offset;

            Err($crate::matcher::MatcherFailure::Error(
              $crate::parse_error::ParseError::new_with_range(
                &context.borrow().get_error_as_string($message, &range),
                range.clone(),
              ),
            ))
          }
          $crate::matcher::MatcherFailure::Error(error) => {
            Err($crate::matcher::MatcherFailure::Error(error))
          }
        }
      }
    )
  };
}
