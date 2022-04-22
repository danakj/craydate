use crate::capi_state::CApiState;
use crate::null_terminated::ToNullTerminatedString;

/// Log a string to the Playdate console, and to stdout.
///
/// Note mostly for internal development: Note that this function may allocate, so must not be
/// called before Playdate initialization.
#[allow(dead_code)]
pub fn log<S: alloc::string::ToString>(s: S) {
  match CApiState::try_get() {
    Some(capi) => {
      let string = s.to_string();
      let vec = string.to_null_terminated_utf8();
      unsafe { capi.csystem.logToConsole.unwrap()(vec.as_ptr()) };
      log_to_stdout("LOG: ");
      log_to_stdout_with_newline(&string);
    }
    None => log_to_stdout_with_newline("ERROR: debug::log() called before debug::initialize()"),
  }
}

/// Prints an error string in red to the Playdate console, and pauses Playdate. Also prints the
/// string to stdout.
///
/// Note mostly for internal development: Note that this function may allocate, so must not be
/// called before Playdate initialization.
pub fn log_error<S: alloc::string::ToString>(s: S) {
  match CApiState::try_get() {
    Some(capi) => {
      let string = s.to_string();
      let vec = string.to_null_terminated_utf8();
      unsafe { capi.csystem.error.unwrap()(vec.as_ptr()) };
      log_to_stdout("ERROR: ");
      log_to_stdout_with_newline(&string);
    }
    None => log_to_stdout_with_newline("ERROR: debug::error() called before debug::initialize()"),
  }
}

/// Log a CString to the simulator console. This function may allocate.
///
/// Note that the simulator console is also sent to stderr.
#[allow(dead_code)]
pub(crate) fn log_c<S: AsRef<str>>(cstr: S) {
  match CApiState::try_get() {
    Some(capi) => unsafe { capi.csystem.logToConsole.unwrap()(cstr.as_ref().as_ptr()) },
    None => log_to_stdout_with_newline("debug::log() called before debug::initialize()"),
  }
}

/// Write a string to stdout, without adding a newline.
///
/// This function will not allocate, and is safe to call from a panic handler.
///
/// This function only works of course when running in a simulator, and if there is support
/// for the current OS. Supported operating systems are:
/// - Windows
#[allow(dead_code)]
pub(crate) fn log_to_stdout<S: AsRef<str>>(s: S) {
  log_bytes_to_stdout(s.as_ref().as_bytes());
}

/// Like log_to_stdout() but adds a newline.
#[allow(dead_code)]
pub(crate) fn log_to_stdout_with_newline<S: AsRef<str>>(s: S) {
  log_bytes_to_stdout(s.as_ref().as_bytes());
  log_bytes_to_stdout(b"\n");
}

#[cfg(target_os = "windows")]
extern "C" {
  fn putchar(c: u8);
  fn _flushall();
}

/// Writes the bytes to stdout, without adding a newline.
pub(crate) fn log_bytes_to_stdout(bytes: &[u8]) {
  for b in bytes {
    unsafe {
      #[cfg(target_os = "windows")]
      putchar(*b);
    }
  }
  unsafe {
    #[cfg(target_os = "windows")]
    _flushall()
  };
}

/// Logs a single byte to stdout.
pub(crate) fn log_byte_to_stdout(byte: u8) {
  unsafe {
    #[cfg(target_os = "windows")]
    putchar(byte);
  }
  unsafe {
    #[cfg(target_os = "windows")]
    _flushall()
  };
}

pub(crate) fn log_usize_to_stdout(num: usize) {
  log_usize_to_stdout_with_radix(num, 10);
}

pub(crate) fn log_usize_to_stdout_with_radix(mut num: usize, radix: usize) {
  const MAX_DIGITS: usize = 20;
  let mut digits: [u8; MAX_DIGITS] = [0; MAX_DIGITS];
  let mut i = 0;
  if num == 0 {
    log_byte_to_stdout('0' as u8)
  } else {
    while num > 0 && i < MAX_DIGITS {
      let digit = (num % radix) as u8;
      num /= radix;
      if digit < 10 {
        digits[i] = '0' as u8 + digit;
      } else {
        digits[i] = 'a' as u8 + (10 - digit);
      }
      i += 1;
    }
    while i > 0 {
      i -= 1;
      log_byte_to_stdout(digits[i]);
    }
  }
}
