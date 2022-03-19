#[macro_export]
macro_rules! Assert {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Assert";
      $crate::Discard!(
        $crate::Optional!(
          $crate::Program!("Assert";
            $matcher,
            $crate::Error!($message),
          )
        )
      )
    )
  };
}
