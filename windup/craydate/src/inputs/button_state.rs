/// The current state of a button, which indicates if the player is holding the button down or not.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonState {
  /// The button is being pressed. The active state of a button.
  Pushed,
  /// The button is not being pressed. The neutral state of a button.
  Released,
}