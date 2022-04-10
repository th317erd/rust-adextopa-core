#[macro_export]
macro_rules! AssertIfNot {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("AssertIf";
      $crate::Optional!(
        $crate::Program!("AssertIf";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Error!($message),
        )
      )
    )
  };
}
