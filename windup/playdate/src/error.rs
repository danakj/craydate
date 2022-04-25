use alloc::string::String;

/// An error performing an operation on a filesystem path.
pub struct FilePathError {
  /// The path of the file operation.
  pub path: String,
  /// The error string reported from Playdate.
  pub playdate: String,
}

/// An error when trying to rename a file or folder, which comes with additional context.
pub struct RenameFilePathError {
  /// The path of the file being renamed.
  pub from_path: String,
  /// The path the file was meant to be moved to.
  pub to_path: String,
  /// The error string reported from Playdate.
  pub playdate: String,
}

/// The Error type for all errors in the playdate crate.
pub enum Error {
  /// A general error which is described by the contained string.
  String(String),
  /// Indicates a file or resource was not found.
  NotFoundError,
  /// An error when trying to use a path, which comes with additional context.
  FilePathError(FilePathError),
  /// An error when trying to rename a file, which comes with additional context.
  RenameFilePathError(RenameFilePathError),
  /// Attempting to load a MIDI file was unsuccessful.
  LoadMidiFileError,
  /// A SoundChannel or SoundSource was already attached and can not be attached again.
  AlreadyAttachedError,
  /// Bitmap dimentions are required to match but they failed to.
  DimensionsDoNotMatch,
  /// An error occured trying to read from a file to play it as audio.
  PlayFileError,
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
impl From<FilePathError> for Error {
  fn from(e: FilePathError) -> Self {
    Error::FilePathError(e)
  }
}
impl From<RenameFilePathError> for Error {
  fn from(e: RenameFilePathError) -> Self {
    Error::RenameFilePathError(e)
  }
}

impl core::fmt::Debug for FilePathError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "FilePathError(path: \"{}\", playdate: \"{}\")",
      self.path, self.playdate
    )
  }
}
impl core::fmt::Debug for RenameFilePathError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "RenameFilePathError(from: \"{}\", to: \"{}\", playdate: \"{}\")",
      self.from_path, self.to_path, self.playdate
    )
  }
}

impl core::fmt::Debug for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::NotFoundError => write!(f, "Error::NotFoundError"),
      Error::FilePathError(file_err) => write!(f, "Error::FilePathError({:?})", file_err,),
      Error::RenameFilePathError(rename_err) => {
        write!(f, "Error::RenameFilePathError({:?})", rename_err)
      }
      Error::AlreadyAttachedError => write!(f, "Error::AlreadyAttachedError"),
      Error::LoadMidiFileError => write!(f, "Error::LoadMidiFileError"),
      Error::DimensionsDoNotMatch => write!(f, "Error::DimensionsDoNotMatch"),
      Error::PlayFileError => write!(f, "Error::PlayFileError"),
      Error::String(e) => write!(f, "Error::String({:?})", e),
    }
  }
}

impl core::fmt::Display for FilePathError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "error operating on the path '{}' (Playdate: {})",
      self.path, self.playdate
    )
  }
}
impl core::fmt::Display for RenameFilePathError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "error renaming the path '{}' to '{}' (Playdate: {})",
      self.from_path, self.to_path, self.playdate
    )
  }
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::NotFoundError => write!(f, "not found"),
      Error::FilePathError(file_err) => file_err.fmt(f),
      Error::RenameFilePathError(rename_err) => rename_err.fmt(f),
      Error::AlreadyAttachedError => write!(f, "already attached"),
      Error::LoadMidiFileError => write!(f, "MIDI file failed to load"),
      Error::DimensionsDoNotMatch => write!(f, "dimensions to not match"),
      Error::PlayFileError => write!(f, "failed to read file to play it as audio"),
      Error::String(e) => e.fmt(f),
    }
  }
}
