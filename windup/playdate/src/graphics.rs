use core::ffi::c_void;
use core::marker::PhantomData;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated;

/// Represents a method for drawing to the display or a bitmap. Similar to a SkPaint in Skia.
#[derive(Debug)]
pub enum LCDColor<'a> {
  /// A single color, which is one of `LCDSolidColor`.
  Solid(LCDSolidColor),
  /// A reference to a 16 byte buffer, the first 8 bytes are 8x8 pixels (each pixel is 1 bit) and the last
  /// 8 bytes are 8x8 masks (each mask is 1 bit) that each defines if the corresponding pixel is used.
  Pattern(&'a LCDPattern),
}

impl LCDColor<'_> {
  /// Returns a usize representation of an LCDColor which can be passed to the Playdate C Api.
  ///
  /// # Safety
  ///
  /// The returned usize for patterns is technically a raw pointer to the LCDPattern array itself. Thus
  /// the caller must ensure that the LCDColor outlives the returned usize. Also, yes really, LCDColor can be
  /// both an enum and a pointer.
  pub(crate) unsafe fn to_c_color(&self) -> usize {
    match self {
      LCDColor::Solid(solid) => solid.0 as usize,
      LCDColor::Pattern(&pattern) => pattern.as_ptr() as usize,
    }
  }
}

/// An opaque handle for a bitmap, which frees the bitmap memory when dropped.
///
/// Get access to the bitmap's data through the `data()` method.
#[derive(Debug)]
pub struct LCDBitmap {
  bitmap_ptr: *mut CLCDBitmap,
  state: &'static CApiState,
}

impl Drop for LCDBitmap {
  fn drop(&mut self) {
    unsafe {
      self.state.graphics.freeBitmap.unwrap()(self.bitmap_ptr);
    }
  }
}

impl LCDBitmap {
  /// Get access to the bitmap's data, including its pixels.
  pub fn data(&self) -> LCDBitmapData {
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut rowbytes: i32 = 0;
    let mut hasmask: i32 = 0;
    let mut data: *mut u8 = core::ptr::null_mut();
    unsafe {
      self.state.graphics.getBitmapData.unwrap()(
        self.bitmap_ptr,
        &mut width,
        &mut height,
        &mut rowbytes,
        &mut hasmask,
        &mut data,
      )
    };
    LCDBitmapData {
      width,
      height,
      rowbytes,
      hasmask,
      data,
      phantom: PhantomData,
    }
  }
}

pub struct LCDBitmapData<'bitmap> {
  width: i32,
  height: i32,
  rowbytes: i32,
  hasmask: i32,
  // TODO: direct access into the bitmap, so does not need to be freed?
  data: *mut u8,
  // Share lifetime of LCDBitmap that generated this.
  phantom: PhantomData<&'bitmap ()>,
}
impl<'bitmap> LCDBitmapData<'bitmap> {
  pub fn width(&self) -> i32 {
    self.width
  }
  pub fn height(&self) -> i32 {
    self.height
  }
  pub fn rowbytes(&self) -> i32 {
    self.rowbytes
  }
  // TODO: is hasmask logically a boolean?
  pub fn hasmask(&self) -> i32 {
    self.hasmask
  }
  /// Gives read acccess to the pixels of the bitmap as an array of bytes. Each byte represents 8 pixels,
  /// where each pixel is a bit. The highest bit is the leftmost pixel, and lowest bit is the rightmost.
  pub fn as_bytes(&self) -> &[u8] {
    unsafe { core::slice::from_raw_parts(self.data, (self.rowbytes * self.height) as usize) }
  }
  /// Gives read-write acccess to the pixels of the bitmap as an array of bytes. Each byte represents 8 pixels,
  /// where each pixel is a bit. The highest bit is the leftmost pixel, and lowest bit is the rightmost.
  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(self.data, (self.rowbytes * self.height) as usize) }
  }
  /// Gives read acccess to the individual pixels of the bitmap.
  pub fn pixels<'data>(&'data self) -> LCDBitmapPixels<'bitmap, 'data> {
    LCDBitmapPixels { data: &self }
  }
  pub fn pixels_mut<'data>(&'data mut self) -> LCDBitmapPixelsMut<'bitmap, 'data> {
    LCDBitmapPixelsMut { data: self }
  }
}

/// Provide shared access to the pixels in an LCDBitmap, through its LCDBitmapData.
pub struct LCDBitmapPixels<'bitmap, 'data> {
  data: &'data LCDBitmapData<'bitmap>,
}
// An impl when LCDBitmapPixels holds a shared reference to LCDBitmapData.
impl LCDBitmapPixels<'_, '_> {
  pub fn get(&self, x: usize, y: usize) -> bool {
    let index = self.data.width as usize * y + x;
    let byte_index = index / 8;
    let bit_index = index % 8;
    (self.data.as_bytes()[byte_index] >> (7 - bit_index)) & 0x1 != 0
  }
}

/// Provide exclusive access to the pixels in an LCDBitmap, through its LCDBitmapData.
pub struct LCDBitmapPixelsMut<'bitmap, 'data> {
  data: &'data mut LCDBitmapData<'bitmap>,
}
// An impl when LCDBitmapPixels holds a mutable reference to LCDBitmapData.
impl LCDBitmapPixelsMut<'_, '_> {
  pub fn get(&self, x: usize, y: usize) -> bool {
    LCDBitmapPixels { data: self.data }.get(x, y)
  }
  pub fn set(&mut self, x: usize, y: usize, new_value: bool) {
    let index = self.data.width as usize * y + x;
    let byte_index = index / 8;
    let bit_index = index % 8;
    if new_value {
      self.data.as_mut_bytes()[byte_index] |= 1u8 << (7 - bit_index);
    } else {
      self.data.as_mut_bytes()[byte_index] &= !(1u8 << (7 - bit_index));
    }
  }
}

#[derive(Debug)]
pub struct Graphics {
  pub(crate) state: &'static CApiState,
}
impl Graphics {
  pub fn clear<'a>(&self, color: LCDColor<'a>) {
    unsafe {
      self.state.graphics.clear.unwrap()(color.to_c_color());
    }
  }

  pub fn set_draw_mode(&self, mode: LCDBitmapDrawMode) {
    unsafe { self.state.graphics.setDrawMode.unwrap()(mode) }
  }

  // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
  pub fn new_bitmap(&self, width: i32, height: i32, bg_color: LCDColor) -> LCDBitmap {
    let bitmap_ptr =
      unsafe { self.state.graphics.newBitmap.unwrap()(width, height, bg_color.to_c_color()) };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }

  pub fn get_bitmap_data<'a>(&self, bitmap: &'a LCDBitmap) -> LCDBitmapData<'a> {
    // This exists to match the API.
    bitmap.data()
  }

  pub fn draw_bitmap(&self, bitmap: &LCDBitmap, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe { self.state.graphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip) }
  }

  pub fn draw_text<S>(&self, text: S, encoding: PDStringEncoding, x: i32, y: i32)
  where
    S: AsRef<str>,
  {
    use crate::null_terminated::ToNullTerminatedString;
    let null_term = text.as_ref().to_null_terminated_utf8();
    let ptr = null_term.as_ptr() as *const c_void;
    let len = null_term.len() as u64;
    unsafe { self.state.graphics.drawText.unwrap()(ptr, len, encoding, x, y) }; // TODO: Return the int from Playdate?
  }

  pub fn copy_frame_buffer_bitmap(&self) -> LCDBitmap {
    let bitmap_ptr = unsafe { self.state.graphics.copyFrameBufferBitmap.unwrap()() };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }
}
