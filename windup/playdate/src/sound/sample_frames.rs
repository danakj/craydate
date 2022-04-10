/// SampleFrames is a unit of time in the sound engine, with 44,100 sample frames per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleFrames(pub(crate) u32);
impl SampleFrames {
  pub fn to_u32(self) -> u32 {
    self.0
  }
}
