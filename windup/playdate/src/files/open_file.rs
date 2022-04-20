use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::NonNull;

use super::file::{last_err, File};
use crate::ctypes::*;
use crate::error::Error;
use crate::format;

/// An open file which can be read from and written to.
///
/// The close() function _must_ be called in order to destroy the `OpenFile` object. Dropping the
/// OpenFile without calling close() will panic/abort.
#[derive(Debug)]
pub(in super) struct OpenFile {
  handle: NonNull<COpenFile>,
  closed: bool,
}
impl OpenFile {
  pub(in super) fn new(handle: NonNull<COpenFile>) -> Self {
    OpenFile {
      handle,
      closed: false,
    }
  }

  /// Read the entire contents of the file.
  pub fn read_file(&self) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    const BUF_SIZE: u32 = 256;
    let mut buf = [0; BUF_SIZE as usize];
    loop {
      let result = unsafe {
        File::fns().read.unwrap()(
          self.handle.as_ptr(),
          buf.as_mut_ptr() as *mut c_void,
          BUF_SIZE,
        )
      };
      let bytes = match result {
        // Reached the end of the file.
        0 => break,
        // Return immediately on an error.
        -1 => Err(format!("error reading from file (Playdate: {}", last_err()))?,
        read_bytes_count => read_bytes_count as usize,
      };
      out.extend(buf[0..bytes].into_iter());
    }
    Ok(out)
  }

  /// Write the entire contents of the file.
  pub fn write_file(&self, contents: &[u8]) -> Result<(), Error> {
    // TODO: This would be needed if we support other operations beyond read/write the whole
    // file.
    // File::fns().seek.unwrap()(self.handle.as_ptr(), 0, playdate_sys::SEEK_SET as i32);

    const BUF_SIZE: usize = 256;
    for buf in contents.chunks(BUF_SIZE) {
      let mut written_from_buffer = 0;
      loop {
        let result = unsafe {
          File::fns().write.unwrap()(
            self.handle.as_ptr().add(written_from_buffer),
            buf.as_ptr() as *const c_void,
            (buf.len() - written_from_buffer) as u32,
          )
        };
        written_from_buffer += match result {
          // Return immediately on an error.
          -1 => Err(format!("error writing to file (Playdate: {}", last_err()))?,
          written_bytes_count => written_bytes_count as usize,
        };
        if written_from_buffer == buf.len() {
          break;
        }
      }
    }
    Ok(())
  }

  /// Close the file. This function _must_ be called in order to destroy the `OpenFile` object.
  ///
  /// Dropping the OpenFile without calling close() will panic/abort.
  pub fn close(mut self) -> Result<(), Error> {
    self.closed = true;
    let result = unsafe { File::fns().close.unwrap()(self.handle.as_ptr()) };
    match result {
      0 => Ok(()),
      _ => Err(format!("error closing file (Playdate: {})", last_err()).into()),
    }
  }
}
impl Drop for OpenFile {
  fn drop(&mut self) {
    if !self.closed {
      crate::debug::log("ERROR: OpenFile dropped without calling close()");
      assert!(self.closed);
    }
  }
}
