use alloc::format;
use core::ptr::NonNull;

use super::bitmap_data::BitmapData;
use super::color::{Color, PixelColor};
use super::unowned_bitmap::UnownedBitmapMut;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::Error;

/// A borrow of a `Bitmap` (or `SharedBitmap`) is held as this type.
///
/// Intentionally not `Copy` as `BitmapRef` can only be referred to as a reference.
#[derive(Debug)]
pub struct BitmapRef {
  ptr: NonNull<CBitmap>,
}
impl BitmapRef {
  /// Construct an BitmapRef from a non-owning pointer.
  pub(crate) fn from_ptr(ptr: NonNull<CBitmap>) -> Self {
    BitmapRef { ptr }
  }

  fn data_and_pixels_ptr(&self) -> (BitmapData, *mut u8) {
    let mut width = 0;
    let mut height = 0;
    let mut rowbytes = 0;
    let mut hasmask = 0;
    let mut pixels = core::ptr::null_mut();
    unsafe {
      // getBitmapData takes a mutable pointer but does not change the data inside it.
      Bitmap::fns().getBitmapData.unwrap()(
        self.cptr() as *mut _,
        &mut width,
        &mut height,
        &mut rowbytes,
        &mut hasmask,
        &mut pixels,
      )
    };
    let data = BitmapData::new(width, height, rowbytes, hasmask);
    (data, pixels)
  }

  /// Loads the image at `path` into the previously allocated `BitmapRef`.
  pub fn load_file(&mut self, path: &str) -> Result<(), Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    unsafe {
      Bitmap::fns().loadIntoBitmap.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        self.cptr_mut(),
        &mut out_err,
      )
    };

    if !out_err.is_null() {
      let result = unsafe { crate::null_terminated::parse_null_terminated_utf8(out_err) };
      match result {
        // A valid error string.
        Ok(err) => Err(format!("load_into_bitmap: {}", err).into()),
        // An invalid error string.
        Err(err) => Err(format!("load_into_bitmap: unknown error ({})", err).into()),
      }
    } else {
      Ok(())
    }
  }

  /// Returns the bitmap's metadata such as its width and height.
  pub fn data(&self) -> BitmapData {
    let (data, _) = self.data_and_pixels_ptr();
    data
  }

  /// Gives read acccess to the pixels of the bitmap as an array of bytes.
  ///
  /// Each byte represents 8 pixels, where each pixel is a bit. The highest bit is the leftmost
  /// pixel, and lowest bit is the rightmost. There are `Bitmap::data().rowbytes` many bytes in each
  /// row, regardless of the number of pixels in a row, which can introduce padding bytes between
  /// rows. For this reason, the `Bitmap::as_pixels()` method is recommended, and easier to use.
  pub fn as_bytes(&self) -> &[u8] {
    let (data, pixels) = self.data_and_pixels_ptr();
    unsafe { core::slice::from_raw_parts(pixels, (data.row_bytes() * data.height()) as usize) }
  }
  /// Gives read-write acccess to the pixels of the bitmap as an array of bytes.
  ///
  /// Each byte represents 8 pixels, where each pixel is a bit. The highest bit is the leftmost
  /// pixel, and lowest bit is the rightmost. There are `Bitmap::data().rowbytes` many bytes in each
  /// row, regardless of the number of pixels in a row, which can introduce padding bytes between
  /// rows. For this reason, the `Bitmap::as_pixels_mit()` method is recommended, and easier to use.
  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    let (data, pixels) = self.data_and_pixels_ptr();
    unsafe { core::slice::from_raw_parts_mut(pixels, (data.row_bytes() * data.height()) as usize) }
  }

  /// Gives read acccess to the individual pixels of the bitmap.
  pub fn as_pixels(&self) -> BitmapPixels {
    let (data, pixels) = self.data_and_pixels_ptr();
    let slice =
      unsafe { core::slice::from_raw_parts(pixels, (data.row_bytes() * data.height()) as usize) };
    BitmapPixels {
      data,
      pixels: slice,
    }
  }
  /// Gives read-write acccess to the individual pixels of the bitmap.
  pub fn as_pixels_mut(&mut self) -> BitmapPixelsMut {
    let (data, pixels) = self.data_and_pixels_ptr();
    let slice = unsafe {
      core::slice::from_raw_parts_mut(pixels, (data.row_bytes() * data.height()) as usize)
    };
    BitmapPixelsMut {
      data,
      pixels: slice,
    }
  }

  /// Clears the bitmap, filling with the given `bg_color`.
  pub fn clear<'a, C: Into<Color<'a>>>(&mut self, bg_color: C) {
    unsafe {
      Bitmap::fns().clearBitmap.unwrap()(self.cptr_mut(), bg_color.into().to_c_color());
    }
  }

  /// Sets a mask image for the given bitmap. The set mask must be the same size as the target
  /// bitmap.
  ///
  /// The mask bitmap is copied, so no reference is held to it. Returns an
  /// Error::DimensionsDoNotMatch if the mask bitmap dimensions do not match with `self`.
  pub fn set_mask_bitmap(&mut self, mask: &BitmapRef) -> Result<(), Error> {
    // Playdate makes a copy of the mask bitmap. It takes a mutable pointer but it only reads from
    // it to do the copy.
    let result =
      unsafe { Bitmap::fns().setBitmapMask.unwrap()(self.cptr_mut(), mask.cptr() as *mut _) };
    match result {
      1 => Ok(()),
      0 => Err(Error::DimensionsDoNotMatch),
      _ => panic!("unknown error result from setBitmapMask"),
    }
  }

  /// The mask bitmap attached to this bitmap.
  ///
  /// Returns the mask bitmap, if one has been attached with `set_mask_bitmap()`, or None.
  pub fn mask_bitmap(&self) -> Option<UnownedBitmapMut> {
    let mask = unsafe {
      // Playdate owns the mask bitmap, and reference a pointer to it. Playdate would free the mask
      // presumably when `self` is freed.
      //
      // getBitmapMask() takes a mutable pointer but does not change the data inside it.
      Bitmap::fns().getBitmapMask.unwrap()(self.cptr() as *mut _)
    };
    Some(UnownedBitmapMut::from_ptr(NonNull::new(mask)?))
  }

  pub(crate) fn cptr(&self) -> *const CBitmap {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CBitmap {
    self.ptr.as_ptr()
  }
  pub(crate) fn copy_non_null(&self) -> NonNull<CBitmap> {
    self.ptr
  }
}

impl alloc::borrow::ToOwned for BitmapRef {
  type Owned = Bitmap;

  /// Makes a deep copy of the `BitmapRef` to produce a new application-owned `Bitmap` that contains
  /// a copy of the pixel data from the `BitmapRef`.
  fn to_owned(&self) -> Self::Owned {
    // copyBitmap() takes a mutable pointer but does not change the data inside it.
    let ptr = unsafe { Bitmap::fns().copyBitmap.unwrap()(self.cptr() as *mut _) };
    Bitmap::from_owned_ptr(NonNull::new(ptr).unwrap())
  }
}

/// A bitmap image.
///
/// The bitmap can be cloned which will make a clone of the pixels as well. The bitmap's pixels data
/// is freed when the bitmap is dropped.
///
/// An `Bitmap` is borrowed as an `&BitmapRef` and all methods of that type are available for
/// `Bitmap as well.
#[derive(Debug)]
pub struct Bitmap {
  /// While BitmapRef is a non-owning pointer, the Bitmap will act as the owner of the bitmap
  /// found within.
  owned: BitmapRef,
}
impl Bitmap {
  /// Construct an Bitmap from an owning pointer.
  pub(crate) fn from_owned_ptr(bitmap_ptr: NonNull<CBitmap>) -> Self {
    Bitmap {
      owned: BitmapRef::from_ptr(bitmap_ptr),
    }
  }

  /// Allocates and returns a new `Bitmap` with pixel dimentions of `width` by `height`. The
  /// bitmap's pixels will be initialized to `bg_color`.
  pub fn new<'a, C: Into<Color<'a>>>(width: i32, height: i32, bg_color: C) -> Bitmap {
    // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
    let bitmap_ptr =
      unsafe { Self::fns().newBitmap.unwrap()(width, height, bg_color.into().to_c_color()) };
    Bitmap::from_owned_ptr(NonNull::new(bitmap_ptr).unwrap())
  }

  /// Returns a new, rotated and scaled Bitmap based on the given `bitmap`.
  pub fn from_bitmap_with_rotation(
    bitmap: &BitmapRef,
    rotation: f32,
    xscale: f32,
    yscale: f32,
  ) -> Bitmap {
    // This function could grow the bitmap by rotating and so it (conveniently?) also returns the
    // alloced size of the new bitmap.  You can get this off the bitmap data more or less if needed.
    let mut _alloced_size: i32 = 0;
    let bitmap_ptr = unsafe {
      // rotatedBitmap() takes a mutable pointer but does not change the data inside it.
      Self::fns().rotatedBitmap.unwrap()(
        bitmap.cptr() as *mut _,
        rotation,
        xscale,
        yscale,
        &mut _alloced_size,
      )
    };
    Bitmap::from_owned_ptr(NonNull::new(bitmap_ptr).unwrap())
  }

  pub fn from_file(path: &str) -> Result<Bitmap, Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    let bitmap_ptr = unsafe {
      Self::fns().loadBitmap.unwrap()(path.to_null_terminated_utf8().as_ptr(), &mut out_err)
    };

    if !out_err.is_null() {
      let result = unsafe { crate::null_terminated::parse_null_terminated_utf8(out_err) };
      match result {
        // A valid error string.
        Ok(err) => Err(format!("load_bitmap: {}", err).into()),
        // An invalid error string.
        Err(err) => Err(format!("load_bitmap: unknown error ({})", err).into()),
      }
    } else {
      assert!(!bitmap_ptr.is_null());
      Ok(Bitmap::from_owned_ptr(NonNull::new(bitmap_ptr).unwrap()))
    }
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}

impl Clone for Bitmap {
  /// Clones the `Bitmap` which includes making a copy of all its pixels.
  fn clone(&self) -> Self {
    use alloc::borrow::ToOwned;
    self.as_ref().to_owned()
  }
}

impl Drop for Bitmap {
  fn drop(&mut self) {
    unsafe {
      Self::fns().freeBitmap.unwrap()(self.cptr_mut());
    }
  }
}

impl core::ops::Deref for Bitmap {
  type Target = BitmapRef;

  fn deref(&self) -> &Self::Target {
    &self.owned
  }
}
impl core::ops::DerefMut for Bitmap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.owned
  }
}
impl core::borrow::Borrow<BitmapRef> for Bitmap {
  fn borrow(&self) -> &BitmapRef {
    self // This calls Deref.
  }
}
impl core::borrow::BorrowMut<BitmapRef> for Bitmap {
  fn borrow_mut(&mut self) -> &mut BitmapRef {
    self // This calls DerefMut.
  }
}
impl AsRef<BitmapRef> for Bitmap {
  fn as_ref(&self) -> &BitmapRef {
    self // This calls Deref.
  }
}
impl AsMut<BitmapRef> for Bitmap {
  fn as_mut(&mut self) -> &mut BitmapRef {
    self // This calls DerefMut.
  }
}

/// Provide readonly access to the pixels in an Bitmap, through its BitmapData.
pub struct BitmapPixels<'bitmap> {
  data: BitmapData,
  pixels: &'bitmap [u8],
}
impl BitmapPixels<'_> {
  pub fn get(&self, x: usize, y: usize) -> PixelColor {
    let byte_index = self.data.row_bytes() as usize * y + x / 8;
    let bit_index = x % 8;
    let bit = (self.pixels[byte_index] >> (7 - bit_index)) & 0x1 == 0x1;
    bit.into()
  }
}

/// Provide mutable access to the pixels in an Bitmap, through its BitmapData.
pub struct BitmapPixelsMut<'bitmap> {
  data: BitmapData,
  pixels: &'bitmap mut [u8],
}
impl BitmapPixelsMut<'_> {
  /// Get the color of the pixel at position `(x, y)`.
  pub fn get(&self, x: usize, y: usize) -> PixelColor {
    let byte_index = self.data.row_bytes() as usize * y + x / 8;
    let bit_index = x % 8;
    let bit = (self.pixels[byte_index] >> (7 - bit_index)) & 0x1 == 0x1;
    bit.into()
  }
  /// Set the pixel at position `(x, y)` to the `PixelColor`.
  pub fn set(&mut self, x: usize, y: usize, new_value: PixelColor) {
    let byte_index = self.data.row_bytes() as usize * y + x / 8;
    let bit_index = x % 8;
    if new_value.to_bit() {
      self.pixels[byte_index] |= 1u8 << (7 - bit_index);
    } else {
      self.pixels[byte_index] &= !(1u8 << (7 - bit_index));
    }
  }
}
