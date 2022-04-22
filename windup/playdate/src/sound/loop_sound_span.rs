use crate::time::{TimeSpan, TimeTicks};

/// A span of time for sound to loop over.
pub enum LoopTimeSpan {
  /// A bounded time span that specifies a start and end time.
  Bounded(TimeSpan),
  /// An unbounded time span that specifies only a start. The span ends at the end of the audio.
  Unbounded(LoopTimeSpanStart),
}
impl LoopTimeSpan {
  /// Returns the start time of the looping time span.
  pub fn start(&self) -> TimeTicks {
    match self {
      Self::Bounded(r) => r.start,
      Self::Unbounded(r) => r.start,
    }
  }
  /// Returns the end time of the looping time span, if there is one.
  pub fn end(&self) -> Option<TimeTicks> {
    match self {
      Self::Bounded(r) => Some(r.end),
      Self::Unbounded(_) => None,
    }
  }
}

#[derive(Debug)]
pub struct LoopTimeSpanStart {
  pub start: TimeTicks,
}
