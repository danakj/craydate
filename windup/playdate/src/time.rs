/// Represents the current device time, which is a monotonically increasing value.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl core::fmt::Debug for TimeTicks {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.to_seconds())
  }
}
impl core::fmt::Debug for TimeDelta {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.to_seconds())
  }
}
