#[macro_export]
macro_rules! FatalIf {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("FatalIf";
      $crate::Optional!(
        $crate::Program!("FatalIf";
          $crate::Discard!($matcher),
          $crate::Panic!($message),
        )
      )
    )
  };
}
