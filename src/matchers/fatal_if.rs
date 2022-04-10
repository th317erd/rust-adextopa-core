#[macro_export]
macro_rules! FatalIf {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("Panic";
      $crate::Optional!(
        $crate::Program!("Panic";
          $crate::Discard!($matcher),
          $crate::Panic!($message),
        )
      )
    )
  };
}
