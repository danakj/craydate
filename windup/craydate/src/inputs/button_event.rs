/// Events which describe changes in state for a button.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonEvent {
  /// The button was pressed.
  ///
  /// It moved from a `Released` to a `Pushed` state.
  Push,
  /// The button stopped being pressed
  ///
  /// It moved from a `Pushed` to a `Released` state.
  Release,
}
