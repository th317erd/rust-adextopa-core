#[macro_export]
macro_rules! ScriptAssignmentExpression {
  () => {
    $crate::Program!("AssignmentExpression";
      $crate::ScriptIdentifier!(),
      $crate::Discard!($crate::Matches!(r"\s*")),
      $crate::Discard!($crate::Equals!("=")),
      $crate::Discard!($crate::Matches!(r"\s*")),
      $crate::ScriptIdentifier!(),
    )
  };
}
