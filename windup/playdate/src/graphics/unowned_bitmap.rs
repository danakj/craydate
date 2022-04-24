use super::bitmap::BitmapRef;
use crate::ctypes::*;

/// A `Bitmap` that is not owned by the application, so can only be used as a borrowed `BitmapRef`.
///
/// A `UnownedBitmapRef` can be cloned which is a shallow clone that produces another borrow on the
/// unowned `Bitmap`.
#[derive(Debug)]
pub struct UnownedBitmapRef<'a> {
  bref: BitmapRef,
  _marker: core::marker::PhantomData<&'a u8>,
}

impl UnownedBitmapRef<'_> {
  /// Construct a UnownedBitmapRef from a non-owning pointer.
  ///
  /// Requires being told the lifetime of the Bitmap this is making a reference to.
  pub(crate) fn from_ptr<'a>(bitmap_ptr: *mut CLCDBitmap) -> UnownedBitmapRef<'a> {
    UnownedBitmapRef {
      bref: BitmapRef::from_ptr(bitmap_ptr),
      _marker: core::marker::PhantomData,
    }
  }

  pub(crate) fn cptr(&self) -> *mut CLCDBitmap {
    self.bref.cptr()
  }
}

impl Clone for UnownedBitmapRef<'_> {
  fn clone(&self) -> Self {
    UnownedBitmapRef::from_ptr(self.cptr())
  }
}

impl core::ops::Deref for UnownedBitmapRef<'_> {
  type Target = BitmapRef;

  fn deref(&self) -> &Self::Target {
    &self.bref
  }
}

impl AsRef<BitmapRef> for UnownedBitmapRef<'_> {
  fn as_ref(&self) -> &BitmapRef {
    self // This calls Deref.
  }
}

/// A mutable `Bitmap` that is not owned by the application, so can only be used as a borrowed
/// `BitmapRef`.
///
/// A `UnownedBitmapRef` can be cloned which is a shallow clone that produces access to another
/// mutable borrow on the unowned `Bitmap`.
pub struct UnownedBitmapMut<'a> {
  bref: UnownedBitmapRef<'a>,
}
impl UnownedBitmapMut<'_> {
  /// Construct a UnownedBitmapMut from a non-owning pointer.
  ///
  /// Requires being told the lifetime of the Bitmap this is making a reference to.
  pub(crate) fn from_ptr<'a>(bitmap_ptr: *mut CLCDBitmap) -> UnownedBitmapMut<'a> {
    UnownedBitmapMut {
      bref: UnownedBitmapRef::from_ptr(bitmap_ptr),
    }
  }

  pub(crate) fn cptr(&self) -> *mut CLCDBitmap {
    self.bref.bref.cptr()
  }
}

impl Clone for UnownedBitmapMut<'_> {
  fn clone(&self) -> Self {
    UnownedBitmapMut::from_ptr(self.cptr())
  }
}

impl core::ops::Deref for UnownedBitmapMut<'_> {
  type Target = BitmapRef;

  fn deref(&self) -> &Self::Target {
    &self.bref.bref
  }
}
impl core::ops::DerefMut for UnownedBitmapMut<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.bref.bref
  }
}

impl AsRef<BitmapRef> for UnownedBitmapMut<'_> {
  fn as_ref(&self) -> &BitmapRef {
    self // This calls Deref.
  }
}
impl AsMut<BitmapRef> for UnownedBitmapMut<'_> {
  fn as_mut(&mut self) -> &mut BitmapRef {
    self // This calls DerefMut.
  }
}
