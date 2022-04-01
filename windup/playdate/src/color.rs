use crate::bitmap::BitmapRef;
use crate::capi_state::CApiState;
use crate::ctypes::*;

const PATTERN_SIZE: usize = 16;

#[derive(Debug)]
#[repr(transparent)]
pub struct Pattern(CLCDPattern);
impl Pattern {
  pub fn new(arr: [u8; PATTERN_SIZE]) -> Pattern {
    Pattern(arr)
  }
  pub fn from_bitmap(bitmap: &BitmapRef, x: i32, y: i32) -> Pattern {
    let mut arr = [0; PATTERN_SIZE];

    let mut c_color: CLCDColor = 0;
    unsafe {
      // The setColorToPattern function wants a `*mut CLCDBitmap`, but it only reads from the bitmap
      // to make a pattern, so we can cast to that from a shared reference to the bitmap.
      let ptr = bitmap.as_bitmap_ptr();
      CApiState::get().cgraphics.setColorToPattern.unwrap()(&mut c_color, ptr, x, y);
      core::ptr::copy_nonoverlapping(c_color as *const u8, arr.as_mut_ptr(), PATTERN_SIZE)
    }

    Pattern(arr)
  }
}

/// Represents a method for drawing to the display or a bitmap. Similar to a SkPaint in Skia.
#[derive(Debug)]
pub enum Color<'a> {
  /// A single color, which is one of `SolidColor`.
  Solid(SolidColor),
  /// A reference to a 16 byte buffer, the first 8 bytes are 8x8 pixels (each pixel is 1 bit) and the last
  /// 8 bytes are 8x8 masks (each mask is 1 bit) that each defines if the corresponding pixel is used.
  Pattern(&'a Pattern),
}

impl From<SolidColor> for Color<'_> {
  fn from(color: SolidColor) -> Self {
    Color::Solid(color)
  }
}

impl<'a> From<&'a Pattern> for Color<'a> {
  fn from(pattern: &'a Pattern) -> Self {
    Color::Pattern(&pattern)
  }
}

impl Color<'_> {
  /// Returns a usize representation of an Color which can be passed to the Playdate C Api.
  ///
  /// # Safety
  ///
  /// The returned usize for patterns is technically a raw pointer to the Pattern array itself. Thus
  /// the caller must ensure that the Color outlives the returned usize. Also, yes really, Color can be
  /// both an enum and a pointer.
  pub(crate) unsafe fn to_c_color(&self) -> usize {
    match self {
      Color::Solid(solid) => solid.0 as usize,
      Color::Pattern(pattern) => pattern.0.as_ptr() as usize,
    }
  }
}
