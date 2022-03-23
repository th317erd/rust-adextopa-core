#[macro_export]
macro_rules! PanicNot {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("PanicNot";
      $crate::Optional!(
        $crate::Program!("PanicNot";
          $crate::Discard!($crate::Not!($matcher)),
          $crate::Fatal!($message),
        )
      )
    )
  };
}
