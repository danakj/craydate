use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::NonNull;

use crate::capi_state::CApiState;
use crate::ctypes::*;

/// An open file which can be read from and written to.
///
/// The close() function _must_ be called in order to destroy the `OpenFile` object. Dropping the
/// OpenFile without calling close() will panic/abort.
#[derive(Debug)]
pub(super) struct OpenFile {
  handle: NonNull<COpenFile>,
  closed: bool,
}
impl OpenFile {
  pub(super) fn new(handle: NonNull<COpenFile>) -> Self {
    OpenFile {
      handle,
      closed: false,
    }
  }

  /// Read the entire contents of the file.
  pub fn read_file(&mut self) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    const BUF_SIZE: u32 = 256;
    let mut buf = [0; BUF_SIZE as usize];
    loop {
      let result = unsafe {
        Self::fns().read.unwrap()(self.cptr_mut(), buf.as_mut_ptr() as *mut c_void, BUF_SIZE)
      };
      let bytes = match result {
        // Reached the end of the file.
        0 => break,
        // Return immediately on an error.
        -1 => None?,
        read_bytes_count => read_bytes_count as usize,
      };
      out.extend(buf[0..bytes].into_iter());
    }
    Some(out)
  }

  /// Write the entire contents of the file, returns if the operation was successful.
  pub fn write_file(&mut self, contents: &[u8]) -> bool {
    // TODO: This would be needed if we support other operations beyond read/write the whole
    // file.
    // Self::fns().seek.unwrap()(self.cptr(), 0, playdate_sys::SEEK_SET as i32);

    const BUF_SIZE: usize = 256;
    for buf in contents.chunks(BUF_SIZE) {
      let mut written_from_buffer = 0;
      loop {
        let result = unsafe {
          Self::fns().write.unwrap()(
            self.cptr_mut().add(written_from_buffer),
            buf.as_ptr() as *const c_void,
            (buf.len() - written_from_buffer) as u32,
          )
        };
        written_from_buffer += match result {
          // Return immediately on an error.
          -1 => return false,
          written_bytes_count => written_bytes_count as usize,
        };
        if written_from_buffer == buf.len() {
          break;
        }
      }
    }
    true
  }

  /// Close the file. This function _must_ be called in order to destroy the `OpenFile` object.
  ///
  /// Dropping the OpenFile without calling close() will panic/abort.
  #[must_use]
  pub fn close(mut self) -> bool {
    self.closed = true;
    let result = unsafe { Self::fns().close.unwrap()(self.cptr_mut()) };
    result == 0
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut COpenFile {
    self.handle.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_file {
    CApiState::get().cfile
  }
}
impl Drop for OpenFile {
  fn drop(&mut self) {
    if !self.closed {
      crate::log::log("ERROR: OpenFile dropped without calling close()");
      assert!(self.closed);
    }
  }
}
