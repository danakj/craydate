#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileTimestamp {
  pub year: i32,
  pub month: i32,
  pub day: i32,
  pub hour: i32,
  pub minute: i32,
  pub second: i32,
}
