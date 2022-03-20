#[macro_export]
macro_rules! ScriptRepeatZeroOrMore {
  () => {
    $crate::Equals!("RepeatZeroOrMore"; "*")
  };
}
