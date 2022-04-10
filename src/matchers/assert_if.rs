#[macro_export]
macro_rules! AssertIf {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("AssertIf";
      $crate::Optional!(
        $crate::Program!("AssertIf";
          $crate::Discard!($matcher),
          $crate::Error!($message),
        )
      )
    )
  };
}
