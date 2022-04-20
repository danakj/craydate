/// The status of the crank input device.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Crank {
  /// When docked, the crank can not be used and as no position.
  Docked,
  Undocked {
    /// The position of the crank in degrees. The angle increases when moved clockwise.
    angle: f32,
    /// The change in position of the crank, in degrees, since the last frame. The angle increases
    /// when moved clockwise, so the change will be negative when moved counter-clockwise.
    change: f32,
  },
}
