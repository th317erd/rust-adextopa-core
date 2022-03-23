#[macro_export]
macro_rules! Assert {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Assert";
      $crate::Optional!(
        $crate::Program!("Assert";
          $crate::Discard!($matcher),
          $crate::Error!($message),
        )
      )
    )
  };
}
