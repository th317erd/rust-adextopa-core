#[macro_export]
macro_rules! ScriptAssignmentExpression {
  () => {
    $crate::Program!("AssignmentExpression";
      $crate::ScriptIdentifier!(),
      $crate::ScriptWSN0!(?),
      $crate::Discard!($crate::Equals!("=")),
      $crate::ScriptWSN0!(?),
      $crate::ScriptPattern!(),
    )
  };
}
