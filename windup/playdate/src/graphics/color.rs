use super::bitmap::BitmapRef;
use crate::capi_state::CApiState;
use crate::ctypes::*;

/// A pattern is 8 bytes representing 8x8 bits of `PixelColor`s followed by 8 bytes representing 8x8
/// bits of a mask to apply the color bits (or to not apply them).
const PATTERN_SIZE: usize = 8 + 8;

/// Represents a method used for operations that draw to the display or a bitmap.
#[derive(Debug)]
pub enum Color<'a> {
  /// A single color, which is one of `SolidColor`.
  Solid(SolidColor),
  /// A reference to a 16 byte buffer, the first 8 bytes are 8x8 pixels (each pixel is 1 bit) and the last
  /// 8 bytes are 8x8 masks (each mask is 1 bit) that each defines if the corresponding pixel is used.
  Pattern(&'a Pattern),
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

/// A pattern is 8x8 bits of data that repeats over a surface. The pattern includes two pieces of
/// data: 8x8 pixels of black/white colors, and an 8x8 mask which is used to draw the pixel color or
/// omit it. Together this forms an 8x8 tri-state of (draw black, draw white, draw nothing).
#[derive(Debug)]
#[repr(transparent)]
pub struct Pattern(CLCDPattern);
impl Pattern {
  /// Creates a `Pattern` from an 8x8 set of colors.
  ///
  /// The pattern is opaque, so all colors will be drawn when the pattern is used. The values are
  /// provided in row-major order, so the first 8 elements make up the first row.
  pub fn new_unmasked(colors: [PixelColor; 8 * 8]) -> Self {
    let mut arr = [0; PATTERN_SIZE];
    let mut bit = 0;
    let mut byte = 0;
    for color in colors {
      let shift = 7 - bit;
      arr[byte] |= (color.to_bit() as u8) << shift;
      bit = (bit + 1) % 8;
      if bit == 0 {
        byte += 1;
      }
    }
    debug_assert_eq!(byte, PATTERN_SIZE / 2);
    debug_assert_eq!(bit, 0);
    for b in &mut arr[PATTERN_SIZE / 2..] {
      *b = 0xff
    }
    Pattern(arr)
  }
  /// Creates a `Pattern` from an 8x8 set of masked colors.
  ///
  /// For each input value, if it's None, then the pattern draws nothing. Otherwise, the pattern
  /// draws the given color. The values are provided in row-major order, so the first 8 elements
  /// make up the first row.
  pub fn new_masked(colors: [Option<PixelColor>; 8 * 8]) -> Self {
    let mut arr = [0; PATTERN_SIZE];
    let mut bit = 0;
    let mut byte = 0;
    for c in colors {
      if let Some(color) = c {
        let shift = 7 - bit;
        arr[byte] |= (color.to_bit() as u8) << shift;
        arr[byte + PATTERN_SIZE / 2] |= 1 << shift;
      }
      bit = (bit + 1) % 8;
      if bit == 0 {
        byte += 1;
      }
    }
    debug_assert_eq!(byte, PATTERN_SIZE / 2);
    debug_assert_eq!(bit, 0);
    Pattern(arr)
  }

  /// Creates a `Pattern` from an array of pattern data, in the same format it's stored internally.
  ///
  /// Each byte of the first 8 bytes represents a row of color values, where for each bit, `1` is
  /// white and `0` is black. Each byte of the last 8 bytes represents a row of a mask, where for
  /// each bit `1` means to draw the pattern's color and `0` means to not draw.
  pub fn from_raw_array(arr: [u8; PATTERN_SIZE]) -> Self {
    Pattern(arr)
  }

  /// Creates a `Pattern` by reading an 8x8 set of pixels from the bitmap and its mask (if it has a
  /// mask).
  pub fn from_bitmap(bitmap: &BitmapRef, x: i32, y: i32) -> Pattern {
    let mut arr = [0; PATTERN_SIZE];

    let mut c_color: CLCDColor = 0;
    unsafe {
      // The setColorToPattern function wants a `*mut CLCDBitmap`, but it only reads from the bitmap
      // to make a pattern, so we can act on a shared `&BitmapRef`.
      CApiState::get().cgraphics.setColorToPattern.unwrap()(&mut c_color, bitmap.cptr(), x, y);
      // The `c_color`, when it's a pattern, contains a pointer to the pattern data. The pattern
      // data here is in static memory that is owned by and will be reused by Playdate. So we must
      // copy the bits out of the pattern pointer, and not free it.
      core::ptr::copy_nonoverlapping(c_color as *const u8, arr.as_mut_ptr(), PATTERN_SIZE);
    }
    Pattern(arr)
  }
}

/// A single pixel's color, either black or white.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct PixelColor(bool);
impl PixelColor {
  /// Returns a bool representation of the color, where black becomes `false` and white becomes
  /// `true`.
  #[inline]
  pub const fn to_bit(self) -> bool {
    use static_assertions::*;
    const_assert_eq!(SolidColor::kColorBlack.0, 0);
    const_assert_eq!(SolidColor::kColorWhite.0, 1);
    self.0
  }

  pub const BLACK: PixelColor = PixelColor(false);
  pub const WHITE: PixelColor = PixelColor(true);
}

impl From<bool> for PixelColor {
  /// Converts from a bool representation to a color. A `false` becomes black, and `true` becomes
  /// white.
  fn from(b: bool) -> Self {
    Self(b)
  }
}

impl core::fmt::Debug for PixelColor {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let s = if self.0 == false { "BLACK" } else { "WHITE" };
    f.debug_tuple("PixelColor").field(&s).finish()
  }
}
