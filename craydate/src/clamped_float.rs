/// A floating point value that is clamped to be within `LOW` and `HIGH`.
#[derive(Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct ClampedFloatInclusive<const LOW: i32, const HIGH: i32>(f32);
impl<const LOW: i32, const HIGH: i32> ClampedFloatInclusive<LOW, HIGH> {
  /// Constructs a new ClampedFloatInclusive.
  pub fn new(f: f32) -> Self {
    ClampedFloatInclusive(f.clamp(LOW as f32, HIGH as f32))
  }

  pub(crate) fn as_mut_ptr(&mut self) -> *mut f32 {
    &mut self.0 as *mut f32
  }

  /// Converts to an `f32` without bounds.
  pub fn to_f32(self) -> f32 {
    self.0
  }
}

impl<const LOW: i32, const HIGH: i32> From<f32> for ClampedFloatInclusive<LOW, HIGH> {
  fn from(f: f32) -> Self {
    ClampedFloatInclusive::new(f)
  }
}
impl<const LOW: i32, const HIGH: i32> From<ClampedFloatInclusive<LOW, HIGH>> for f32 {
  fn from(c: ClampedFloatInclusive<LOW, HIGH>) -> Self {
    c.to_f32()
  }
}

impl<const LOW: i32, const HIGH: i32> Default for ClampedFloatInclusive<LOW, HIGH> {
  fn default() -> Self {
    Self(LOW as f32)
  }
}

impl<const LOW: i32, const HIGH: i32> core::ops::Add for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self::new(self.0.add(rhs.0))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Sub for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self::new(self.0.sub(rhs.0))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Mul for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self::new(self.0.mul(rhs.0))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Div for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
    Self::new(self.0.div(rhs.0))
  }
}

impl<const LOW: i32, const HIGH: i32> core::ops::Add<f32> for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn add(self, rhs: f32) -> Self::Output {
    Self::new(self.0.add(rhs))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Sub<f32> for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn sub(self, rhs: f32) -> Self::Output {
    Self::new(self.0.sub(rhs))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Mul<f32> for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self::Output {
    Self::new(self.0.mul(rhs))
  }
}
impl<const LOW: i32, const HIGH: i32> core::ops::Div<f32> for ClampedFloatInclusive<LOW, HIGH> {
  type Output = Self;

  fn div(self, rhs: f32) -> Self::Output {
    Self::new(self.0.div(rhs))
  }
}

impl<const LOW: i32, const HIGH: i32> core::fmt::Display for ClampedFloatInclusive<LOW, HIGH> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}
