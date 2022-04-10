use crate::time::{TimeDelta, TimeTicks};

pub enum LoopSoundRange {
  Bounded(SoundRange),
  Unbounded(SoundRangeStart),
}
impl LoopSoundRange {
  pub(crate) fn start(&self) -> TimeTicks {
    match self {
      Self::Bounded(r) => r.start,
      Self::Unbounded(r) => r.start,
    }
  }
  pub(crate) fn end(&self) -> Option<TimeTicks> {
    match self {
      Self::Bounded(r) => Some(r.end),
      Self::Unbounded(_) => None,
    }
  }
}

#[derive(Debug)]
pub struct SoundRangeStart {
  pub start: TimeTicks,
}

#[derive(Debug)]
pub struct SoundRange {
  pub start: TimeTicks,
  pub end: TimeTicks,
}

#[derive(Debug)]
pub struct SignedSoundRange {
  pub start: TimeDelta,
  pub end: TimeDelta,
}
