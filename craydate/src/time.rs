use core::cell::Cell;

use crate::ctypes::*;

/// Represents the current device time, which is a monotonically increasing value.
///
/// At this time the highest resolution available is milliseconds, so callers that need a raw
/// value should normally use `total_whole_milliseconds()`. However it is always preferable to
/// retain the TimeTicks type instead of unwrapping a primitive type from it.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeTicks(pub(crate) u32); // Stores milliseconds.
impl TimeTicks {
  pub fn from_milliseconds(millis: u32) -> Self {
    TimeTicks(millis)
  }
  pub fn from_seconds_lossy(sec: f32) -> Self {
    TimeTicks((sec * 1000f32) as u32)
  }

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

  /// Constructs a time from the number of sound sample frames.
  pub(crate) fn from_sample_frames(frames: u32) -> Self {
    TimeTicks(frames * 1000 / crate::sound::SAMPLE_FRAMES_PER_SEC as u32)
  }
  /// Returns the time in the number of sound sample frames.
  pub(crate) fn to_sample_frames(self) -> u32 {
    self.total_whole_milliseconds() * crate::sound::SAMPLE_FRAMES_PER_SEC as u32 / 1000
  }
}

/// The difference between two TimeTicks.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i32); // Stores milliseconds.
impl TimeDelta {
  /// Constructs a TimeDelta that represents the given number of days.
  pub const fn from_days(h: i32) -> Self {
    Self::from_hours(h * 24)
  }
  /// Constructs a TimeDelta that represents the given number of hours.
  pub const fn from_hours(h: i32) -> Self {
    Self::from_minutes(h * 60)
  }
  /// Constructs a TimeDelta that represents the given number of minutes.
  pub const fn from_minutes(m: i32) -> Self {
    Self::from_seconds(m * 60)
  }
  /// Constructs a TimeDelta that represents the given number of seconds.
  pub const fn from_seconds(s: i32) -> Self {
    TimeDelta(s * 1000)
  }
  /// Constructs a TimeDelta that represents the given number of milliseconds.
  pub const fn from_milliseconds(s: i32) -> Self {
    TimeDelta(s)
  }

  pub fn from_seconds_lossy(sec: f32) -> Self {
    TimeDelta((sec * 1000f32) as i32)
  }

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

  /// Constructs a time delta from the number of sound sample frames.
  #[allow(dead_code)]  // Not currently used.
  pub(crate) fn from_sample_frames(frames: i32) -> Self {
    TimeDelta(frames * 1000 / crate::sound::SAMPLE_FRAMES_PER_SEC)
  }
  /// Returns the time delta in the number of sound sample frames.
  pub(crate) fn to_sample_frames(self) -> i32 {
    self.total_whole_milliseconds() * crate::sound::SAMPLE_FRAMES_PER_SEC / 1000
  }
}

impl core::ops::Add<TimeDelta> for TimeTicks {
  type Output = TimeTicks;

  fn add(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0.checked_add(rhs.0 as u32).unwrap())
    } else {
      TimeTicks(self.0.checked_sub((-rhs.0) as u32).unwrap())
    }
  }
}
impl core::ops::Sub<TimeDelta> for TimeTicks {
  type Output = TimeTicks;

  fn sub(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0.checked_sub(rhs.0 as u32).unwrap())
    } else {
      TimeTicks(self.0.checked_add((-rhs.0) as u32).unwrap())
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
  csystem: &'static CSystemApi,
  active_marker: &'a Cell<bool>,
}
impl<'a> HighResolutionTimer<'a> {
  pub(crate) fn new(csystem: &'static CSystemApi, active_marker: &'a Cell<bool>) -> Self {
    active_marker.set(true);
    HighResolutionTimer {
      csystem,
      active_marker,
    }
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

/// Represents a wall-clock time.
///
/// The time can be affected by changing the clock on the device, or by clock drift. As such this
/// time is not guaranteed to increase monotonically. This can be useful for displaying a clock, if
/// combined with timezone data, but should not be used for tracking elapsed time. `TimeTicks`
/// should be used for that instead.
///
/// Similar to the standard library type `std::time::SystemTime`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WallClockTime(pub(crate) u32);
impl WallClockTime {
  /// The epoch represents the time at midnight (hour 0), January 1, 2000.
  ///
  /// Note that this is different from the well-known unix epoch which is 1970.
  #[allow(dead_code)]
  pub const PLAYDATE_EPOCH: WallClockTime = WallClockTime(0);
}

impl core::ops::Add<TimeDelta> for WallClockTime {
  type Output = TimeTicks;

  fn add(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0.checked_add(rhs.0 as u32).unwrap())
    } else {
      TimeTicks(self.0.checked_sub((-rhs.0) as u32).unwrap())
    }
  }
}
impl core::ops::Sub<TimeDelta> for WallClockTime {
  type Output = TimeTicks;

  fn sub(self, rhs: TimeDelta) -> Self::Output {
    if rhs.0 >= 0 {
      TimeTicks(self.0.checked_sub(rhs.0 as u32).unwrap())
    } else {
      TimeTicks(self.0.checked_add((-rhs.0) as u32).unwrap())
    }
  }
}

impl core::ops::Sub<WallClockTime> for WallClockTime {
  type Output = TimeDelta;

  fn sub(self, rhs: WallClockTime) -> Self::Output {
    if self > rhs {
      let positive_val = self.0 - rhs.0;
      TimeDelta(positive_val as i32)
    } else {
      let positive_val = rhs.0 - self.0;
      TimeDelta(-(positive_val as i32))
    }
  }
}

/// A span of time with an absolute (unsigned) start and end.
#[derive(Debug)]
pub struct TimeSpan {
  pub start: TimeTicks,
  pub end: TimeTicks,
}

/// A span of time with a relative (signed) start and end.
#[derive(Debug)]
pub struct RelativeTimeSpan {
  pub start: TimeDelta,
  pub end: TimeDelta,
}
