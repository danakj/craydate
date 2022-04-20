/// Prints a string to the Playdate console, as well as to stdout.
pub fn log<S: alloc::string::ToString>(s: S) {
  crate::debug::log(&s.to_string())
}

/// Prints an error string in red to the Playdate console, and pauses Playdate. Also prints the
/// string to stdout.
pub fn log_error<S: alloc::string::ToString>(s: S) {
  crate::debug::error(&s.to_string());
}
