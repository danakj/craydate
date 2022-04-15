/// A range of midi notes, which can include all notes, a single note, or a contiguous set of notes.
pub enum MidiNoteRange {
  /// All midi notes are included.
  All,
  /// Only a single midi note is included.
  Single(f32),
  /// A contiguous set of notes is included.
  StartEnd(f32, f32),
}
impl MidiNoteRange {
    pub(crate) fn to_start_end(&self) -> (f32, f32) {
        match self {
            Self::All => (f32::MIN, f32::MAX),
            Self::Single(s) => (*s, *s),
            Self::StartEnd(start, end) => (*start, *end)
        }
    }
}