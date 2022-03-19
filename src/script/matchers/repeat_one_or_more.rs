#[macro_export]
macro_rules! ScriptRepeatOneOrMore {
  () => {
    $crate::Equals!("RepeatOneOrMore"; "+")
  };
}
