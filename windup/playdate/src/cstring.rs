use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct CString {
  v: Vec<u8>,
}
impl CString {
  pub fn new() -> CString {
    CString { v: Vec::new() }
  }

  pub fn from_vec<V: Into<Vec<u8>>>(v: V) -> Option<CString> {
    let vec: Vec<u8> = v.into();
    let bytes = vec.as_slice();

    // TODO: Use memchr()
    for i in 0..bytes.len() {
      if bytes[i] == 0 {
        return None;
      }
    }

    let v = unsafe {
      let num_bytes_without_nul = bytes.len();
      let mut v = Vec::with_capacity(num_bytes_without_nul + 1);
      core::ptr::copy_nonoverlapping(bytes.as_ptr(), v.as_mut_ptr(), num_bytes_without_nul);
      *v.as_mut_ptr().add(num_bytes_without_nul) = 0;
      v.set_len(num_bytes_without_nul + 1);
      v
    };

    Some(CString { v })
  }

  pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> CString {
    CString { v: bytes.into() }
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.v.as_ptr()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.v.as_slice()
  }
  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    self.v.as_mut_slice()
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
  #[inline]
  #[must_use]
  pub unsafe fn from_bytes_with_nul_unchecked_mut(s: &mut [u8]) -> &mut CStr {
    // SAFETY: Safe to cast because Cstr is repr(transparent) so they have the same byte
    // representation.
    &mut *(s as *mut [u8] as *mut CStr)
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.0.as_ptr()
  }

  pub fn to_bytes_with_nul(&self) -> &[u8] {
    &self.0
  }

  pub fn to_bytes(&self) -> &[u8] {
    &self.0[..self.0.len() - 1]
  }

  pub fn to_owned(&self) -> CString {
    unsafe { CString::from_bytes_unchecked(&self.0) }
  }
}

impl Default for CString {
  fn default() -> Self {
    CString::new()
  }
}

impl Borrow<CStr> for CString {
  fn borrow(&self) -> &CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes()) }
  }
}
impl BorrowMut<CStr> for CString {
  fn borrow_mut(&mut self) -> &mut CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked_mut(self.as_mut_bytes()) }
  }
}
impl Deref for CString {
  type Target = CStr;
  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}
impl DerefMut for CString {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut()
  }
}
impl AsRef<CStr> for CString {
  fn as_ref(&self) -> &CStr {
    self
  }
}
impl AsMut<CStr> for CString {
  fn as_mut(&mut self) -> &mut CStr {
    self
  }
}
impl AsRef<CStr> for CStr {
  fn as_ref(&self) -> &CStr {
    self
  }
}
impl AsMut<CStr> for CStr {
  fn as_mut(&mut self) -> &mut CStr {
    self
  }
}
