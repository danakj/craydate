/// The set of input buttons.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Button {
  /// The up arrow on the directional pad.
  Up,
  /// The down arrow on the directional pad.
  Down,
  /// The left arrow on the directional pad.
  Left,
  /// The right arrow on the directional pad.
  Right,
  /// The B button.
  B,
  /// The A button.
  A,
}