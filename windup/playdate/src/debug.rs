use crate::ctypes::*;
use crate::CStr;

pub fn _log(system: &'static CSystem, bytes: &[u8]) {
  unsafe { system.logToConsole.unwrap()(CStr::from_bytes_with_nul(bytes).unwrap().as_ptr()) }
}
