#[derive(Debug, Default)]
pub struct StereoVolume {
  pub left: f32,  // TODO: Replace with some Type<f32> that clamps the value to 0-1.
  pub right: f32, // TODO: Replace with some Type<f32> that clamps the value to 0-1.
}
impl StereoVolume {
  pub fn new(left: f32, right: f32) -> Self {
    StereoVolume { left, right }
  }
  pub fn zero() -> Self {
    Self::new(0.0, 0.0)
  }
  pub fn one() -> Self {
    Self::new(1.0, 1.0)
  }
}
