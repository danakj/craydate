use std::error::Error;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, PlaydateBuildError>;

#[derive(Debug)]
pub enum PlaydateBuildError {
  IOError(std::io::Error),
  PdxCompilerError(String),
}

impl Error for PlaydateBuildError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::IOError(e) => Some(e),
      _ => None,
    }
  }
}

impl Display for PlaydateBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IOError(e) => write!(f, "{}", e),
      Self::PdxCompilerError(s) => write!(f, "{}", s),
    }
  }
}

impl From<std::io::Error> for PlaydateBuildError {
  fn from(e: std::io::Error) -> Self {
    Self::IOError(e)
  }
}
