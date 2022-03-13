use alloc::vec::Vec;

#[derive(Hash)]
pub struct CString {
  v: Vec<u8>,
}
impl CString {
  pub fn new(s: &str) -> Option<CString> {
    let bytes = s.as_bytes();

    // TODO: Use memchr()
    for i in 0..bytes.len() {
      if bytes[i] == 0 {
        return None;
      }
    }

    let v = unsafe {
      let mut v = Vec::with_capacity(s.len() + 1);
      core::ptr::copy_nonoverlapping(bytes.as_ptr(), v.as_mut_ptr(), bytes.len());
      v[s.len()] = 0;
      v.set_len(v.len() + 1);
      v
    };

    Some(CString { v })
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.v.as_ptr()
  }
}
impl core::ops::Deref for CString {
  type Target = CStr;
  fn deref(&self) -> &Self::Target {
    unsafe { CStr::from_bytes_with_nul_unchecked(&self.v) }
  }
}

#[repr(transparent)]
#[derive(Hash)]
pub struct CStr([u8]);
impl CStr {
  pub fn from_bytes_with_nul(s: &[u8]) -> Option<&CStr> {
    // TODO: Use memchr()
    for i in 0..s.len() - 1 {
      if s[i] == 0 {
        return None;
      }
    }
    if s[s.len() - 1] != 0 {
      return None;
    }

    Some(unsafe { Self::from_bytes_with_nul_unchecked(s) })
  }
  #[inline]
  #[must_use]
  pub unsafe fn from_bytes_with_nul_unchecked(s: &[u8]) -> &CStr {
    // SAFETY: Safe to cast because Cstr is repr(transparent) so they have the same byte
    // representation.
    &*(s as *const [u8] as *const CStr)
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.0.as_ptr()
  }
}
