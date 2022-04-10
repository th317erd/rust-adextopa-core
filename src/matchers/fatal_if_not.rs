#[macro_export]
macro_rules! FatalIfNot {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("PanicNot";
      $crate::Optional!(
        $crate::Program!("PanicNot";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Panic!($message),
        )
      )
    )
  };
}
