use crate::ctypes::*;
use core::cell::Cell;

/// Represents the current device time, which is a monotonically increasing value.
///
/// At this time the highest resolution available is milliseconds, so callers that need a raw
/// value should normally use `total_whole_milliseconds()`. However it is always preferable to
/// retain the TimeTicks type instead of unwrapping a primitive type from it.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeTicks(u32);
impl TimeTicks {
  // Returns the number of hours passed in the time, truncating any non-whole hours.
  pub fn total_whole_hours(&self) -> u32 {
    self.0 / (1000 * 60 * 60)
  }
  // Returns the number of minutes passed in the time, truncating any non-whole minutes.
  pub fn total_whole_minutes(&self) -> u32 {
    self.0 / (1000 * 60)
  }
  // Returns the number of seconds passed in the time, truncating any non-whole seconds.
  pub fn total_whole_seconds(&self) -> u32 {
    self.0 / 1000
  }
  // Returns the number of milliseconds passed in the time, truncating any non-whole milliseconds.
  pub fn total_whole_milliseconds(&self) -> u32 {
    self.0
  }

  /// Returns the time represented as seconds.
  pub fn to_seconds(self) -> f32 {
    (self.0 as f32) / 1000f32
  }
}

/// The difference between two TimeTicks.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i32);
impl TimeDelta {
  // Returns the number of hours in the delta, truncating any non-whole hours.
  pub fn total_whole_hours(&self) -> i32 {
    self.0 / (1000 * 60 * 60)
  }
  // Returns the number of minutes in the delta, truncating any non-whole minutes.
  pub fn total_whole_minutes(&self) -> i32 {
    self.0 / (1000 * 60)
  }
  // Returns the number of seconds in the delta, truncating any non-whole seconds.
  pub fn total_whole_seconds(&self) -> i32 {
    self.0 / 1000
  }
  // Returns the number of milliseconds in the delta, truncating any non-whole milliseconds.
  pub fn total_whole_milliseconds(&self) -> i32 {
    self.0
  }

  /// Returns the time delta represented as seconds.
  pub fn to_seconds(self) -> f32 {
    (self.0 as f32) / 1000f32
  }
}

impl core::ops::Add<TimeDelta> for TimeTicks {
  type Output = TimeTicks;

  fn add(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0 + rhs.0 as u32)
    } else {
      TimeTicks(self.0 - (-rhs.0) as u32)
    }
  }
}
impl core::ops::Sub<TimeDelta> for TimeTicks {
  type Output = TimeTicks;

  fn sub(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0 - rhs.0 as u32)
    } else {
      TimeTicks(self.0 + (-rhs.0) as u32)
    }
  }
}

impl core::ops::Sub<TimeTicks> for TimeTicks {
  type Output = TimeDelta;

  fn sub(self, rhs: TimeTicks) -> Self::Output {
    if self > rhs {
      let positive_val = self.0 - rhs.0;
      TimeDelta(positive_val as i32)
    } else {
      let positive_val = rhs.0 - self.0;
      TimeDelta(-(positive_val as i32))
    }
  }
}

impl From<u32> for TimeTicks {
  fn from(u: u32) -> Self {
    TimeTicks(u)
  }
}
impl From<i32> for TimeDelta {
  fn from(i: i32) -> Self {
    TimeDelta(i)
  }
}

impl core::fmt::Display for TimeTicks {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{} seconds", self.to_seconds())
  }
}
impl core::fmt::Display for TimeDelta {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{} seconds", self.to_seconds())
  }
}

/// The system's high resolution timer. There is only one timer available in the system.
///
#[derive(Debug)]
/// The timer is meant for measuring short times, and the longer it runs the more lossy it becomes.
pub struct HighResolutionTimer<'a> {
  csystem: &'static CSystem,
  active_marker: &'a Cell<bool>
}
impl<'a> HighResolutionTimer<'a> {
  pub(crate) fn new(csystem: &'static CSystem, active_marker: &'a Cell<bool>) -> Self {
    active_marker.set(true);
    HighResolutionTimer { csystem, active_marker }
  }

  fn elapsed(&self) -> f32 {
    unsafe { self.csystem.getElapsedTime.unwrap()() }
  }

  pub fn elapsed_seconds_lossy(&self) -> f32 {
    self.elapsed()
  }

  /// Return the elapsed number of millisecnds since the timer started.
  ///
  /// As a u32 can only represent 2147 seconds, the returned value will simply stop
  /// increasing if it reaches u32::MAX. While the HighResolutionTimer is not meant
  /// for tracking long times, `elapsed_seconds_lossy()` can track elapsed times that
  /// exceed u32::MAX.
  pub fn elapsed_microseconds(&self) -> u32 {
    let seconds = self.elapsed();

    // `f32::trunc()` not in `no_std`.
    let seconds_whole = unsafe { core::intrinsics::truncf32(seconds) };
    let seconds_fract = seconds - seconds_whole;

    // We pull out the fractional part before multiplying to avoid losing precision in it due to the
    // whole number part.
    let micros_from_whole = (seconds_whole * 1000000f32) as u32;
    let micros_from_fract = (seconds_fract * 1000000f32) as u32;

    micros_from_whole.checked_add(micros_from_fract).unwrap_or(u32::MAX)
  }
}

impl Drop for HighResolutionTimer<'_> {
    fn drop(&mut self) {
        self.active_marker.set(false)
    }
}