// #[macro_export]
// macro_rules! AssertIfNot {
//   ($matcher:expr, $message:expr) => {
//     $crate::ProxyChildren!("AssertIfNot";
//       $crate::Optional!(
//         $crate::Program!("AssertIfNot";
//           $crate::Discard!($crate::Not!($matcher)),
//           $crate::Error!($message),
//         )
//       )
//     )
//   };
// }

#[macro_export]
macro_rules! AssertIfNot {
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
            let start = context.borrow().offset.start;

            Ok($crate::matcher::MatcherSuccess::Token(
              $crate::matchers::error::new_error_token_with_range(
                context.clone(),
                $message,
                &SourceRange::new(start, start),
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
