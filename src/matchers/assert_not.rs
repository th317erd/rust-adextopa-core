#[macro_export]
macro_rules! AssertNot {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Assert";
      $crate::Discard!(
        $crate::Optional!(
          $crate::Program!("Assert";
            $crate::Not!($matcher),
            $crate::Error!($message),
          )
        )
      )
    )
  };
}
