use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

pub struct String {
  data: Vec<u8>,
}
impl String {
  pub fn new() -> String {
    String { data: Vec::new() }
  }

  pub fn from_utf8(from: Vec<u8>) -> Result<String, core::str::Utf8Error> {
    // If from_utf8 returns a `&str`, it means the Vec contains valid utf8. Since &str must point to the
    // Vec, we steal the Vec as the String buffer to avoid copying.
    match core::str::from_utf8(&from) {
      Ok(_) => Ok(String { data: from }),
      Err(e) => Err(e)
    }
  }

  pub fn with_capacity(capacity: usize) -> String {
    String {
      data: Vec::with_capacity(capacity),
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.data
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn clear(&mut self) {
    self.data.clear()
  }

  pub unsafe fn as_mut_vec(&mut self) -> &Vec<u8> {
    &self.data
  }
}

impl Deref for String {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    unsafe { core::str::from_utf8_unchecked(self.data.as_slice()) }
  }
}
impl DerefMut for String {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { core::str::from_utf8_unchecked_mut(self.data.as_mut_slice()) }
  }
}

impl Borrow<str> for String {
  fn borrow(&self) -> &str {
    self
  }
}
impl BorrowMut<str> for String {
  fn borrow_mut(&mut self) -> &mut str {
    self
  }
}

impl AsRef<str> for String {
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}
impl AsRef<[u8]> for String {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    return self.data.as_slice();
  }
}
impl AsMut<str> for String {
  #[inline]
  fn as_mut(&mut self) -> &mut str {
    self
  }
}

impl core::fmt::Debug for String {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    (&**self).fmt(f) // Forward to str.
  }
}
impl Default for String {
  fn default() -> Self {
    String::new()
  }
}
impl core::cmp::PartialEq for String {
  fn eq(&self, other: &Self) -> bool {
    (&**self).eq(&**other) // Forward to str.
  }
}
impl core::cmp::Eq for String {}
impl core::cmp::PartialOrd for String {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    (&**self).partial_cmp(&**other) // Forward to str.
  }
}
impl core::cmp::Ord for String {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    (&**self).cmp(&**other) // Forward to str.
  }
}
impl core::hash::Hash for String {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    (&**self).hash(state)
  }
}

impl core::str::FromStr for String {
  type Err = core::convert::Infallible;
  #[inline]
  fn from_str(s: &str) -> Result<String, Self::Err> {
    Ok(String::from(s))
  }
}

impl From<&str> for String {
  fn from(s: &str) -> Self {
    let byte_len = s.as_bytes().len();
    let mut out = String::with_capacity(byte_len);
    unsafe {
      // SAFETY: The String buffer was allocated with byte_len capacity above.
      core::ptr::copy_nonoverlapping(s.as_ptr(), out.data.as_mut_ptr(), byte_len);
      // SAFETY: We have initialized all `byte_len` bytes of the String buffer above.
      out.data.set_len(byte_len);
    }
    out
  }
}
impl From<&mut str> for String {
  fn from(s: &mut str) -> Self {
    String::from(s as &str)
  }
}
