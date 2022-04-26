/// A filesystem timestamp, which can represent when a file or folder was last accessed, modified,
/// etc.
/// 
/// The values here derived from
/// <https://sdk.play.date/1.10.0/Inside%20Playdate.html#f-file.modtime>.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePathTimestamp {
  /// The timestamp's year.
  pub year: i32,
  /// The timestamp's month within the year, from 1 to 12.
  pub month: i32,
  /// The timestamp's day within the month, from 1 to 31.
  pub day: i32,
  /// The timestamp's hour within the day, from 0 to 23.
  pub hour: i32,
  /// The timestamp's minute within the hour, from 0 to 59.
  pub minute: i32,
  /// The timestamp's seconds within the minute, normally from 0 to 59. Can be 60 on a leap second.
  pub second: i32,
}
