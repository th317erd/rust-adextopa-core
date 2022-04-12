#[macro_export]
macro_rules! FatalIfNot {
  ($matcher:expr, $message:expr) => {
    $crate::ProxyChildren!("FatalIfNot";
      $crate::Optional!(
        $crate::Program!("FatalIfNot";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Panic!($message),
        )
      )
    )
  };
}
