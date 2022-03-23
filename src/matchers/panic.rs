#[macro_export]
macro_rules! Panic {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("Panic";
      $crate::Discard!(
        $crate::Optional!(
          $crate::Program!("Panic";
            $matcher,
            $crate::Fatal!($message),
          )
        )
      )
    )
  };
}
