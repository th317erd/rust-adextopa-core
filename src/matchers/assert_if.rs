// #[macro_export]
// macro_rules! AssertIf {
//   ($matcher:expr, $message:expr) => {
//     $crate::ProxyChildren!("AssertIf";
//       $crate::Optional!(
//         $crate::Program!("AssertIf";
//           $crate::Discard!($matcher),
//           $crate::Error!($message),
//         )
//       )
//     )
//   };
// }

#[macro_export]
macro_rules! AssertIf {
  ($matcher:expr, $message:expr) => {
    $crate::Map!(
      $matcher,
      |token, context, _| {
        // If the match succeeded, then throw an error
        let _token = token.borrow();
        let range = _token.get_matched_range();

        Ok($crate::matcher::MatcherSuccess::Token(
          $crate::matchers::error::new_error_token_with_range(context.clone(), $message, &range),
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
