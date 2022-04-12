#[macro_export]
macro_rules! AssertIfNot {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("AssertIfNot";
      $crate::Optional!(
        $crate::Program!("AssertIfNot";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Error!($message),
        )
      )
    )
  };
}
