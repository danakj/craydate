/// A range of MIDI notes, which can include all notes, a single note, or a contiguous set of notes.
pub enum MidiNoteRange {
  /// All midi notes are included.
  All,
  /// Only a single MIDI note is included. The value is a MIDI note number, which is between 0 and
  /// 127.
  ///
  /// See: <https://syntheway.com/MIDI_Keyboards_Middle_C_MIDI_Note_Number_60_C4.htm>
  ///
  /// TODO: Support MIDI string notation (e.g. "Db3").
  Single(u8),
  /// A contiguous set of notes is included. The values are MIDI note numers, which are between 0 and 127.
  ///
  /// See: <https://syntheway.com/MIDI_Keyboards_Middle_C_MIDI_Note_Number_60_C4.htm>
  ///
  /// TODO: Support MIDI string notation (e.g. "Db3").
  StartEnd(u8, u8),
}
impl MidiNoteRange {
  pub(crate) fn to_start_end(&self) -> (u8, u8) {
    match self {
      Self::All => (u8::MIN, u8::MAX),
      Self::Single(s) => (*s, *s),
      Self::StartEnd(start, end) => (*start, *end),
    }
  }
}
