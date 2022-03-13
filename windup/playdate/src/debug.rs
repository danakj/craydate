use crate::ctypes::*;
use crate::CStr;

pub unsafe fn _log(system: &'static CSystem, bytes: &[u8]) {
  system.logToConsole.unwrap()(CStr::from_bytes_with_nul(bytes).unwrap().as_ptr())
}
