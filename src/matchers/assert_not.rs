#[macro_export]
macro_rules! AssertNot {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Assert";
      $crate::Optional!(
        $crate::Program!("Assert";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Error!($message),
        )
      )
    )
  };
}
