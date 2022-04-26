use super::super::volume::Volume;

/// A MIDI note which is played as part of a `SequenceTrack` in a `SequenceTrack`.
#[derive(Debug)]
pub struct TrackNote {
  /// The MIDI note number, which is between 0 and 127.
  ///
  /// See: <https://syntheway.com/MIDI_Keyboards_Middle_C_MIDI_Note_Number_60_C4.htm>
  ///
  /// TODO: Support MIDI string notation (e.g. "Db3").
  pub midi_note: u8,
  /// Velocity indicates how hard the key was struck when the note was played, which usually
  /// corresponds to the note's loudness.
  pub velocity: Volume,
}
impl Default for TrackNote {
  fn default() -> Self {
    Self {
      midi_note: 60,
      velocity: Volume::one(),
    }
  }
}

/// A MIDI note resolved to a note number and with a length, which is played as part of a `SequenceTrack`
/// in a `SequenceTrack`.
#[derive(Debug)]
pub struct ResolvedTrackNote {
  /// The MIDI note number, which is between 0 and 127.
  ///
  /// See: <https://syntheway.com/MIDI_Keyboards_Middle_C_MIDI_Note_Number_60_C4.htm>
  pub midi_note: u8,
  /// Velocity indicates how hard the key was struck when the note was played, which usually
  /// corresponds to the note's loudness.
  pub velocity: Volume,
  /// The length of the note in steps, not time. That is, the time follows the `Sequence`’s tempo.
  pub length: u32,
}
