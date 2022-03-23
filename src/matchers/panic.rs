#[macro_export]
macro_rules! Panic {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Panic";
      $crate::Optional!(
        $crate::Program!("Panic";
          $crate::Discard!($matcher),
          $crate::Fatal!($message),
        )
      )
    )
  };
}
