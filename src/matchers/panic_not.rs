#[macro_export]
macro_rules! PanicNot {
  ($matcher:expr, $message:expr) => {
    $crate::Flatten!("PanicNot";
      $crate::Discard!(
        $crate::Optional!(
          $crate::Program!("PanicNot";
            $crate::Not!($matcher),
            $crate::Fatal!($message),
          )
        )
      )
    )
  };
}
