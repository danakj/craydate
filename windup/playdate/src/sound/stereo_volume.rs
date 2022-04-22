use crate::clamped_float::ClampedFloatInclusive;

/// A volume with two channels: left and right.
#[derive(Debug, Default)]
pub struct StereoVolume {
  /// The left channel volume, a value between 0 and 1.
  pub left: ClampedFloatInclusive<0, 1>,
  /// The right channel volume, a value between 0 and 1.
  pub right: ClampedFloatInclusive<0, 1>,
}
impl StereoVolume {
  /// Constructs a new StereoVolume from `f32` volume levels that will be clamped to within 0 and 1.
  pub fn new(left: f32, right: f32) -> Self {
    StereoVolume {
      left: left.into(),
      right: right.into(),
    }
  }
  /// Constructs a new StereoVolume with the given channel volume levels.
  pub fn with_clamped(
    left: ClampedFloatInclusive<0, 1>,
    right: ClampedFloatInclusive<0, 1>,
  ) -> Self {
    StereoVolume { left, right }
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
