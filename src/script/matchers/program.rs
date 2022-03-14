#[macro_export]
macro_rules! ScriptProgram {
  () => {{
    $crate::Loop!("Program"; $crate::ScriptAssignmentExpression!())
  }};
}
