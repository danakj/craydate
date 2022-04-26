use std::error::Error;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, CraydateBuildError>;

#[derive(Debug)]
pub enum CraydateBuildError {
  IOError(std::io::Error),
  PdxCompilerError(String),
  String(String),
}

impl Error for CraydateBuildError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::IOError(e) => Some(e),
      _ => None,
    }
  }
}

impl Display for CraydateBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::IOError(e) => write!(f, "{}", e),
      Self::PdxCompilerError(s) => write!(f, "{}", s),
      Self::String(s) => write!(f, "{}", s),
    }
  }
}

impl From<std::io::Error> for CraydateBuildError {
  fn from(e: std::io::Error) -> Self {
    Self::IOError(e)
  }
}
impl From<String> for CraydateBuildError {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}
