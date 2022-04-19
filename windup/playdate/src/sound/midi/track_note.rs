#[derive(Debug)]
pub struct TrackNote {
  /// The length of the note in steps, not time - ​that is, it follows the sequence’s tempo.
  pub length: u32,
  // TODO: Support MIDI string notation (e.g. "Db3").
  pub midi_note: f32,
  pub velocity: f32,
}
impl Default for TrackNote {
  fn default() -> Self {
    Self {
      length: 1,
      midi_note: 0.0,
      velocity: 1.0,
    }
  }
}
