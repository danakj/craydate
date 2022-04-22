use alloc::string::String;

/// The Error type for all errors in the playdate crate.
pub enum Error {
  /// A general error which is described by the contained string.
  String(String),
  /// Indicates a file or resource was not found.
  NotFoundError,
  /// Attempting to load a MIDI file was unsuccessful.
  LoadMidiFileError,
  /// A SoundChannel or SoundSource was already attached and can not be attached again.
  AlreadyAttachedError,
}
impl From<String> for Error {
  fn from(s: String) -> Self {
    Error::String(s)
  }
}
impl From<&str> for Error {
  fn from(s: &str) -> Self {
    Error::String(s.into())
  }
}
impl From<&mut str> for Error {
  fn from(s: &mut str) -> Self {
    Error::String(s.into())
  }
}

impl core::fmt::Debug for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::NotFoundError => write!(f, "Error(NotFoundError)"),
      Error::AlreadyAttachedError => write!(f, "Error(AlreadyAttachedError)"),
      Error::LoadMidiFileError => write!(f, "Error(LoadMidiFileError"),
      Error::String(e) => write!(f, "Error(String({:?}))", e),
    }
  }
}
impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::NotFoundError => write!(f, "not found"),
      Error::AlreadyAttachedError => write!(f, "already attached"),
      Error::LoadMidiFileError => write!(f, "MIDI file failed to load"),
      Error::String(e) => write!(f, "{}", e),
    }
  }
}
