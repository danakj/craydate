use crate::clamped_float::ClampedFloatInclusive;

/// A volume with two channels: left and right.
#[derive(Debug, Default)]
pub struct StereoVolume {
  /// The left channel volume, a value between 0 and 1.
  pub left: Volume,
  /// The right channel volume, a value between 0 and 1.
  pub right: Volume,
}
impl StereoVolume {
  /// Constructs a new StereoVolume from `f32` volume levels that will be clamped to within 0 and 1.
  pub fn new(left: f32, right: f32) -> Self {
    StereoVolume {
      left: Volume::new(left),
      right: Volume::new(right),
    }
  }
  /// Constructs a new StereoVolume with the given already-clamped channel volume levels.
  pub fn with_clamped(
    left: ClampedFloatInclusive<0, 1>,
    right: ClampedFloatInclusive<0, 1>,
  ) -> Self {
    StereoVolume {
      left: Volume::with_clamped(left),
      right: Volume::with_clamped(right),
    }
  }
  /// Constructs a StereoVolume with both channels set to 0.
  pub fn zero() -> Self {
    Self::new(0.0, 0.0)
  }
  /// Constructs a StereoVolume with both channels set to 1.
  pub fn one() -> Self {
    Self::new(1.0, 1.0)
  }
}

/// A volume, which is a value between 0 and 1. Setting it to a value outside the valid range will
/// result in the value being clamped to within 0 and 1.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Volume(ClampedFloatInclusive<0, 1>);
impl Volume {
  /// Constructs a new Volume from a `f32` volume level that will be clamped to within 0 and 1.
  pub fn new(vol: f32) -> Self {
    Volume(vol.into())
  }
  /// Constructs a new Volume with the given already-clamped volume level.
  pub fn with_clamped(vol: ClampedFloatInclusive<0, 1>) -> Self {
    Volume(vol)
  }
  /// Constructs a Volume set to 0.
  pub fn zero() -> Self {
    Self::new(0.0)
  }
  /// Constructs a Volume set to 1.
  pub fn one() -> Self {
    Self::new(1.0)
  }

  pub(crate) fn as_mut_ptr(&mut self) -> *mut f32 {
    self.0.as_mut_ptr()
  }

  /// Converts to an `f32` without bounds.
  pub fn to_f32(self) -> f32 {
    self.0.to_f32()
  }
}

impl From<f32> for Volume {
  fn from(f: f32) -> Self {
    Volume::new(f)
  }
}
impl From<Volume> for f32 {
  fn from(v: Volume) -> Self {
    v.0.into()
  }
}
impl core::fmt::Display for Volume {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.0.fmt(f)
  }
}

impl core::ops::Add for Volume {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self::with_clamped(self.0.add(rhs.0))
  }
}
impl core::ops::Sub for Volume {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self::with_clamped(self.0.sub(rhs.0))
  }
}
impl core::ops::Mul for Volume {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self::with_clamped(self.0.mul(rhs.0))
  }
}
impl core::ops::Div for Volume {
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
    Self::with_clamped(self.0.div(rhs.0))
  }
}

impl core::ops::Add<f32> for Volume {
  type Output = Self;

  fn add(self, rhs: f32) -> Self::Output {
    Self::with_clamped(self.0.add(rhs))
  }
}
impl core::ops::Sub<f32> for Volume {
  type Output = Self;

  fn sub(self, rhs: f32) -> Self::Output {
    Self::with_clamped(self.0.sub(rhs))
  }
}
impl core::ops::Mul<f32> for Volume {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self::Output {
    Self::with_clamped(self.0.mul(rhs))
  }
}
impl core::ops::Div<f32> for Volume {
  type Output = Self;

  fn div(self, rhs: f32) -> Self::Output {
    Self::with_clamped(self.0.div(rhs))
  }
}
