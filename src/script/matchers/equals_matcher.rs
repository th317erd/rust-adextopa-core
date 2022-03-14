#[macro_export]
macro_rules! ScriptEqualsMatcher {
  () => {
    $crate::Program!(
      "EqualsMatcher";
      $crate::Discard!($crate::Equals!("=")),
      $crate::Discard!($crate::Matches!(r"\s*")),
      $crate::ScriptIdentifier!(),
    )
  };
}
