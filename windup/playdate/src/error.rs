use alloc::string::String;

pub enum Error {
  BorrowError(core::cell::BorrowError),
  BorrowMutError(core::cell::BorrowMutError),
  NotFoundError,
  LoadMidiFileError,
  String(String),
}
impl From<core::cell::BorrowError> for Error {
  fn from(e: core::cell::BorrowError) -> Self {
    Error::BorrowError(e)
  }
}
impl From<core::cell::BorrowMutError> for Error {
  fn from(e: core::cell::BorrowMutError) -> Self {
    Error::BorrowMutError(e)
  }
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
      Error::BorrowError(e) => write!(f, "Error(BorrowError({:?}))", e),
      Error::BorrowMutError(e) => write!(f, "Error(BorrowMutError({:?}))", e),
      Error::NotFoundError => write!(f, "Error(NotFoundError)"),
      Error::LoadMidiFileError => write!(f, "Error(LoadMidiFileError"),
      Error::String(e) => write!(f, "Error(String({:?}))", e),
    }
  }
}
impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::BorrowError(e) => write!(f, "{}", e),
      Error::BorrowMutError(e) => write!(f, "{}", e),
      Error::NotFoundError => write!(f, "not found"),
      Error::LoadMidiFileError => write!(f, "MIDI file failed to load"),
      Error::String(e) => write!(f, "{}", e),
    }
  }
}
