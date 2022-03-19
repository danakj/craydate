use core::ffi::c_void;

use crate::api::Error;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::String;

/// Represents a method for drawing to the display or a bitmap. Similar to a SkPaint in Skia.
#[derive(Debug)]
pub enum LCDColor<'a> {
  /// A single color, which is one of `LCDSolidColor`.
  Solid(LCDSolidColor),
  /// A reference to a 16 byte buffer, the first 8 bytes are 8x8 pixels (each pixel is 1 bit) and the last
  /// 8 bytes are 8x8 masks (each mask is 1 bit) that each defines if the corresponding pixel is used.
  Pattern(&'a LCDPattern),
}

impl From<LCDSolidColor> for LCDColor<'_> {
  fn from(color: LCDSolidColor) -> Self {
    LCDColor::Solid(color)
  }
}

impl<'a> From<&'a LCDPattern> for LCDColor<'a> {
  fn from(pattern: &'a LCDPattern) -> Self {
    LCDColor::Pattern(&pattern)
  }
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

/// A bitmap image.
///
/// The bitmap can be cloned which will make a clone of the pixels as well. The bitmap's pixels
/// data is freed when the bitmap is dropped.
#[derive(Debug)]
pub struct LCDBitmap {
  bitmap_ptr: *mut CLCDBitmap,
  state: &'static CApiState,
}

impl Clone for LCDBitmap {
  fn clone(&self) -> Self {
    LCDBitmap {
      bitmap_ptr: unsafe { self.state.cgraphics.copyBitmap.unwrap()(self.bitmap_ptr) },
      state: self.state,
    }
  }
}

impl Drop for LCDBitmap {
  fn drop(&mut self) {
    unsafe {
      self.state.cgraphics.freeBitmap.unwrap()(self.bitmap_ptr);
    }
  }
}

impl LCDBitmap {
  fn data_and_pixels_ptr(&self) -> (LCDBitmapData, *mut u8) {
    let mut width = 0;
    let mut height = 0;
    let mut rowbytes = 0;
    let mut hasmask = 0;
    let mut pixels = core::ptr::null_mut();
    unsafe {
      self.state.cgraphics.getBitmapData.unwrap()(
        self.bitmap_ptr,
        &mut width,
        &mut height,
        &mut rowbytes,
        &mut hasmask,
        &mut pixels,
      )
    };
    let data = LCDBitmapData {
      width,
      height,
      rowbytes,
      hasmask,
    };
    (data, pixels)
  }

  /// Get access to the bitmap's data.
  pub fn data(&self) -> LCDBitmapData {
    let (data, _) = self.data_and_pixels_ptr();
    data
  }

  /// Gives read acccess to the pixels of the bitmap as an array of bytes. Each byte represents 8 pixels,
  /// where each pixel is a bit. The highest bit is the leftmost pixel, and lowest bit is the rightmost.
  pub fn as_bytes(&self) -> &[u8] {
    let (data, pixels) = self.data_and_pixels_ptr();
    unsafe { core::slice::from_raw_parts(pixels, (data.rowbytes * data.height) as usize) }
  }
  /// Gives read-write acccess to the pixels of the bitmap as an array of bytes. Each byte represents 8 pixels,
  /// where each pixel is a bit. The highest bit is the leftmost pixel, and lowest bit is the rightmost.
  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    let (data, pixels) = self.data_and_pixels_ptr();
    unsafe { core::slice::from_raw_parts_mut(pixels, (data.rowbytes * data.height) as usize) }
  }
  /// Gives read acccess to the individual pixels of the bitmap.
  pub fn pixels(&self) -> LCDBitmapPixels {
    let (data, pixels) = self.data_and_pixels_ptr();
    let slice =
      unsafe { core::slice::from_raw_parts(pixels, (data.rowbytes * data.height) as usize) };
    LCDBitmapPixels {
      data,
      pixels: slice,
    }
  }
  /// Gives read-write acccess to the individual pixels of the bitmap.
  pub fn pixels_mut(&mut self) -> LCDBitmapPixelsMut {
    let (data, pixels) = self.data_and_pixels_ptr();
    let slice =
      unsafe { core::slice::from_raw_parts_mut(pixels, (data.rowbytes * data.height) as usize) };
    LCDBitmapPixelsMut {
      data,
      pixels: slice,
    }
  }

  /// Clears the bitmap, filling with the given `bgcolor`.
  pub fn clear<'a, C>(&mut self, bgcolor: C)
  where
    LCDColor<'a>: From<C>,
  {
    unsafe {
      self.state.cgraphics.clearBitmap.unwrap()(
        self.bitmap_ptr,
        LCDColor::<'a>::from(bgcolor).to_c_color(),
      );
    }
  }

  pub(crate) unsafe fn get_bitmap_ptr(&self) -> *const CLCDBitmap {
    self.bitmap_ptr
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LCDBitmapData {
  width: i32,
  height: i32,
  rowbytes: i32,
  hasmask: i32,
}
impl LCDBitmapData {
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
}

/// Provide readonly access to the pixels in an LCDBitmap, through its LCDBitmapData.
pub struct LCDBitmapPixels<'bitmap> {
  data: LCDBitmapData,
  pixels: &'bitmap [u8],
}
impl LCDBitmapPixels<'_> {
  pub fn get(&self, x: usize, y: usize) -> bool {
    let byte_index = self.data.rowbytes as usize * y + x / 8;
    let bit_index = x % 8;
    (self.pixels[byte_index] >> (7 - bit_index)) & 0x1 != 0
  }
}

/// Provide mutable access to the pixels in an LCDBitmap, through its LCDBitmapData.
pub struct LCDBitmapPixelsMut<'bitmap> {
  data: LCDBitmapData,
  pixels: &'bitmap mut [u8],
}
impl LCDBitmapPixelsMut<'_> {
  pub fn get(&self, x: usize, y: usize) -> bool {
    LCDBitmapPixels {
      data: self.data,
      pixels: self.pixels,
    }
    .get(x, y)
  }
  pub fn set(&mut self, x: usize, y: usize, new_value: bool) {
    let byte_index = self.data.rowbytes as usize * y + x / 8;
    let bit_index = x % 8;
    if new_value {
      self.pixels[byte_index] |= 1u8 << (7 - bit_index);
    } else {
      self.pixels[byte_index] &= !(1u8 << (7 - bit_index));
    }
  }
}

#[derive(Debug)]
pub struct Graphics {
  pub(crate) state: &'static CApiState,
}
impl Graphics {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Graphics { state }
  }

  /// Clears the entire display, filling it with `color`.
  pub fn clear<'a, C>(&mut self, color: C)
  where
    LCDColor<'a>: From<C>,
  {
    unsafe {
      self.state.cgraphics.clear.unwrap()(LCDColor::<'a>::from(color).to_c_color());
    }
  }

  /// Sets the background color shown when the display is offset or for clearing dirty areas
  /// in the sprite system.
  pub fn set_background_color(&mut self, color: LCDSolidColor) {
    unsafe {
      self.state.cgraphics.setBackgroundColor.unwrap()(color);
    }
  }

  /// Manually flushes the current frame buffer out to the display. This function is automatically
  /// called after each pass through the run loop, so there shouldn’t be any need to call it
  /// yourself.
  pub fn display(&mut self) {
    unsafe {
      self.state.cgraphics.display.unwrap()();
    }
  }

  // TODO: getDebugBitmap
  // TODO: getDisplayFrame
  // TODO: getDisplayFrameBitmap
  // TODO: getFrame

  /// Returns a copy the contents of the working frame buffer as a bitmap.
  pub fn copy_frame_buffer_bitmap(&self) -> LCDBitmap {
    let bitmap_ptr = unsafe { self.state.cgraphics.copyFrameBufferBitmap.unwrap()() };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }

  /// After updating pixels in the buffer returned by `get_frame()`, you must tell the graphics
  /// system which rows were updated. This function marks a contiguous range of rows as updated
  /// (e.g., `mark_updated_rows(0, LCD_ROWS - 1)` tells the system to update the entire display).
  /// Both "start" and "end" are included in the range.
  pub fn mark_updated_rows(&mut self, start: i32, end: i32) {
    unsafe { self.state.cgraphics.markUpdatedRows.unwrap()(start, end) }
  }

  /// Offsets the origin point for all drawing calls to x, y (can be negative).
  pub fn set_draw_offset(&mut self, dx: i32, dy: i32) {
    unsafe { self.state.cgraphics.setDrawOffset.unwrap()(dx, dy) }
  }

  // TODO: setSpriteDrawFunction
  // TODO: setColorToPattern
  // TODO: all the graphics->video functions
  // TODO: pushContext/popContext
  //       do these funcs need to borrow the LCDBitmap while it's "on the stack"??
  // TODO: setStencil: what's the lifetime with the stencil bitmap???

  /// Sets the mode used for drawing bitmaps. Note that text drawing uses bitmaps, so this
  /// affects how fonts are displayed as well.
  pub fn set_draw_mode(&mut self, mode: LCDBitmapDrawMode) {
    unsafe { self.state.cgraphics.setDrawMode.unwrap()(mode) }
  }

  // TODO: checkMaskCollision

  /// Draws the bitmap to the screen.
  ///
  /// The bitmap's upper-left corner is positioned at location (`x`, `y`), and the contents have
  /// the `flip` orientation applied.
  pub fn draw_bitmap(&mut self, bitmap: &LCDBitmap, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe { self.state.cgraphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip) }
  }

  /// Draws the bitmap to the screen, scaled by `xscale` and `yscale`.
  ///
  /// /// The bitmap's upper-left corner is positioned at location (`x`, `y`). Note that flip is not
  /// available when drawing scaled bitmaps but negative scale values will achieve the same effect.
  pub fn draw_scaled_bitmap(
    &mut self,
    bitmap: &LCDBitmap,
    x: i32,
    y: i32,
    xscale: f32,
    yscale: f32,
  ) {
    unsafe {
      self.state.cgraphics.drawScaledBitmap.unwrap()(bitmap.bitmap_ptr, x, y, xscale, yscale)
    }
  }

  /// Draws the bitmap to the screen, scaled by `xscale` and `yscale` then rotated by `degrees` with
  /// its center as given by proportions `centerx` and `centery` at (`x`, `y`); that is: if
  /// `centerx` and `centery` are both 0.5 the center of the image is at (`x`, `y`), if `centerx`
  /// and `centery` are both 0 the top left corner of the image (before rotation) is at (`x`, `y`),
  /// etc.
  pub fn draw_rotated_bitmap(
    &mut self,
    bitmap: &LCDBitmap,
    x: i32,
    y: i32,
    degrees: f32,
    centerx: f32,
    centery: f32,
    xscale: f32,
    yscale: f32,
  ) {
    unsafe {
      self.state.cgraphics.drawRotatedBitmap.unwrap()(
        bitmap.bitmap_ptr,
        x,
        y,
        degrees,
        centerx,
        centery,
        xscale,
        yscale,
      )
    }
  }

  /// Draws the bitmap to the screen with its upper-left corner at location (`x`, `y`) tiled inside
  /// a `width` by `height` rectangle.
  pub fn draw_tiled_bitmap(
    &mut self,
    bitmap: &LCDBitmap,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flip: LCDBitmapFlip,
  ) {
    unsafe {
      self.state.cgraphics.tileBitmap.unwrap()(bitmap.bitmap_ptr, x, y, width, height, flip)
    }
  }

  pub fn load_bitmap(&self, path: &str) -> Result<LCDBitmap, Error> {
    use crate::null_terminated::ToNullTerminatedString;
    let path = path.to_null_terminated_utf8().as_ptr();
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image).
    // However, future calls will overwrite the previous out_err and trying to free it
    // via system->realloc crashes (likely because the pointer wasn't alloc'd by us).
    // This probably (hopefully??) means that we don't need to free it.
    let bitmap_ptr = unsafe { self.state.cgraphics.loadBitmap.unwrap()(path, &mut out_err) };

    if bitmap_ptr.is_null() {
      if !out_err.is_null() {
        unsafe {
          let result = crate::null_terminated::parse_null_terminated_utf8(out_err);
          if let Ok(out_err) = result {
            return Err(String::from("LoadBitmap: ") + &out_err)?;
          }
        }
      }

      return Err("LoadBitmap: unknown error")?;
    }

    Ok(LCDBitmap {
      bitmap_ptr,
      state: self.state,
    })
  }

  // TODO: loadIntoBitmap (what happens if image doesn't exist?)

  /// Allocates and returns a new `width` by `height` `LCDBitmap` filled with `bg_color`.
  pub fn new_bitmap<'a, C>(&self, width: i32, height: i32, bg_color: C) -> LCDBitmap
  where
    LCDColor<'a>: From<C>,
  {
    // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
    let bitmap_ptr = unsafe {
      self.state.cgraphics.newBitmap.unwrap()(
        width,
        height,
        LCDColor::<'a>::from(bg_color).to_c_color(),
      )
    };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }

  /// Returns a new, rotated and scaled LCDBitmap based on the given `bitmap`.
  pub fn new_rotated_bitmap(
    &self,
    bitmap: &LCDBitmap,
    rotation: f32,
    xscale: f32,
    yscale: f32,
  ) -> LCDBitmap {
    // This function could grow the bitmap by rotating and so it (conveniently?) also returns the
    // alloced size of the new bitmap.  You can get this off the bitmap data more or less if needed.
    let mut _alloced_size: i32 = 0;
    LCDBitmap {
      bitmap_ptr: unsafe {
        self.state.cgraphics.rotatedBitmap.unwrap()(
          bitmap.bitmap_ptr,
          rotation,
          xscale,
          yscale,
          &mut _alloced_size,
        )
      },
      state: self.state,
    }
  }

  // TODO: setBitmapMask
  // TODO: getBitmapMask

  // TODO: getTableBitmap
  // TODO: loadBitmapTable
  // TODO: loadIntoBitmapTable
  // TODO: newBitmapTable

  pub fn draw_text(&mut self, text: &str, encoding: PDStringEncoding, x: i32, y: i32) {
    use crate::null_terminated::ToNullTerminatedString;
    let null_term = text.to_null_terminated_utf8();
    let ptr = null_term.as_ptr() as *const c_void;
    let len = null_term.len() as u64;
    unsafe { self.state.cgraphics.drawText.unwrap()(ptr, len, encoding, x, y) }; // TODO: Return the int from Playdate?
  }

  /// Draws the current FPS on the screen at the given (`x`, `y`) coordinates.
  pub fn draw_fps(&mut self, x: i32, y: i32) {
    // This function is part of Playdate CSystem, not CGraphics, but it's a function that draws
    // something to the screen, so its behaviour is more clear when part of the Graphics type.
    unsafe { self.state.csystem.drawFPS.unwrap()(x, y) }
  }
}
