//! Traits for converting to and from null-terminated UTF-encoded C strings.

use alloc::vec::Vec;

pub trait ToNullTerminatedString {
  /// Produce a utf8-encoded buffer that is terminated with a null.
  fn to_null_terminated_utf8(&self) -> Vec<u8>;
}

impl ToNullTerminatedString for &str {
  fn to_null_terminated_utf8(&self) -> Vec<u8> {
    let num_bytes_without_nul = self.as_bytes().len();
    let mut v = Vec::with_capacity(num_bytes_without_nul + 1);
    unsafe {
      core::ptr::copy_nonoverlapping(self.as_ptr(), v.as_mut_ptr(), num_bytes_without_nul);
      *v.as_mut_ptr().add(num_bytes_without_nul) = 0;
      v.set_len(num_bytes_without_nul + 1);
    }
    v
  }
}
impl ToNullTerminatedString for alloc::string::String {
  fn to_null_terminated_utf8(&self) -> Vec<u8> {
    (&**self).to_null_terminated_utf8()
  }
}

/// A simple implementation of strlen() from the C standard library.
///
/// # Safety
///
/// The input pointer must be to an allocation that contains a null, otherwise this will
/// read off the end of the allocation which introduces Undefined Behaviour.
#[inline]
unsafe fn strlen(s: *const u8) -> usize {
  let mut isize = 0;
  while *s.offset(isize) != 0 {
    isize += 1;
  }
  return isize as usize;
}

/// Parse a buffer of unknown size, without an attached lifetime, into a `&str`. The buffer must be able to
/// be converted to a UTF-8 string, or an error would be returned.
///
/// # Safety
///
/// This function assigns a lifetime to the returned `&str` and the caller must verify that
/// the chosen lifetime is correct.
///
/// For strings coming from "const char** outerr" in the playdate api, these strings appear to be
/// written into a fixed static buffer where future errors will overwrite the first.
pub unsafe fn parse_null_terminated_utf8<'a>(
  p: *const u8,
) -> Result<&'a str, core::str::Utf8Error> {
  let slice = {
    let num_bytes_without_nul = strlen(p);
    core::slice::from_raw_parts::<'a>(p, num_bytes_without_nul)
  };
  core::str::from_utf8(slice)
}