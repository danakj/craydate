use crate::ctypes::*;
use crate::CStr;

struct SystemRef(&'static CSystem);
unsafe impl Sync for SystemRef {}

static mut SYSTEM: Option<SystemRef> = None;

pub fn initialize(system: &'static CSystem) {
  unsafe { SYSTEM = Some(SystemRef(system)) }
}

/// SAFETY: The `bytes` must be null-terminated.
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn _log(bytes: &[u8]) {
  let system: &'static CSystem = SYSTEM.unwrap().0;
  match CStr::from_bytes_with_nul(bytes) {
    Some(cstr) => unsafe { system.logToConsole.unwrap()(cstr.as_ptr()) },
    None => puts(b"Invalid bytes given to log()"),
  }
}


#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn log(bytes: &[u8]) {
  extern "C" {
    fn puts(c: *const u8);
    fn _flushall();
  }

  match CStr::from_bytes_with_nul(bytes) {
    Some(cstr) => unsafe { puts(cstr.as_ptr()) },
    None => unsafe { puts(b"Invalid bytes given to log()".as_ptr()) },
  }
  unsafe { _flushall() };
}
