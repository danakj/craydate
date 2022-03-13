use crate::ctypes::*;
use crate::CStr;

struct SystemRef(&'static CSystem);
unsafe impl Sync for SystemRef {}

static mut SYSTEM: Option<SystemRef> = None;

pub fn initialize(system: &'static CSystem) {
  unsafe { SYSTEM = Some(SystemRef(system)) }
  log(b"debug::log initialized.\0");
}

#[allow(dead_code)]
pub fn log(bytes: &[u8]) {
  let maybe_system: Option<&'static CSystem> = unsafe { SYSTEM.as_ref().map(|r| r.0) };
  match CStr::from_bytes_with_nul(bytes) {
    Some(cstr) => {
      match maybe_system {
        Some(system) => unsafe { system.logToConsole.unwrap()(cstr.as_ptr()) },
        None => log_bytes_to_stdout(b"debug::log() called before debug::initialize()\n"),
      }
    }
    None => log_bytes_to_stdout(b"Invalid bytes given to log()\n"),
  }
}

#[cfg(target_os = "windows")]
extern "C" {
  fn putchar(c: u8);
  fn _flushall();
}

/// Logs the bytes to stdout directly. They should not be null-terminated.
pub fn log_bytes_to_stdout(bytes: &[u8]) {
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
pub fn log_byte_to_stdout(byte: u8) {
  unsafe {
    #[cfg(target_os = "windows")]
    putchar(byte);
  }
  unsafe {
    #[cfg(target_os = "windows")]
    _flushall()
  };
}

pub fn log_usize_to_stdout(num: usize) {
  log_usize_to_stdout_with_radix(num, 10);
}

pub fn log_usize_to_stdout_with_radix(mut num: usize, radix: usize) {
  const MAX_DIGITS: usize = 20;
  let mut digits: [u8; MAX_DIGITS] = [0; MAX_DIGITS];
  let mut i = 0;
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
