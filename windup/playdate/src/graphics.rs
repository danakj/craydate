use core::ffi::c_void;
use core::marker::PhantomData;

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
      self.state.cgraphics.freeBitmap.unwrap()(self.bitmap_ptr);
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
      self.state.cgraphics.getBitmapData.unwrap()(
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

  pub(crate) unsafe fn get_mut_ptr(&self) -> *mut CLCDBitmap {
    self.bitmap_ptr
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
    let byte_index = self.data.rowbytes as usize * y + x / 8;
    let bit_index = x % 8;
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
    let byte_index = self.data.rowbytes as usize * y + x / 8;
    let bit_index = x % 8;
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
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Graphics { state }
  }

  /// Clears the entire display, filling it with `color`.
  pub fn clear<'a, C>(&self, color: C)
  where
    C: Into<LCDColor<'a>>,
  {
    unsafe {
      self.state.cgraphics.clear.unwrap()(color.into().to_c_color());
    }
  }

  /// Sets the background color shown when the display is offset or for clearing dirty areas
  /// in the sprite system.
  pub fn set_background_color(&self, color: LCDSolidColor) {
    unsafe {
      self.state.cgraphics.setBackgroundColor.unwrap()(color);
    }
  }

  /// Manually flushes the current frame buffer out to the display. This function is automatically
  /// called after each pass through the run loop, so there shouldn’t be any need to call it
  /// yourself.
  pub fn display(&self) {
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
  pub fn mark_updated_rows(&self, start: i32, end: i32) {
    unsafe { self.state.cgraphics.markUpdatedRows.unwrap()(start, end) }
  }

  /// Offsets the origin point for all drawing calls to x, y (can be negative).
  pub fn set_draw_offset(&self, dx: i32, dy: i32) {
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
  pub fn set_draw_mode(&self, mode: LCDBitmapDrawMode) {
    unsafe { self.state.cgraphics.setDrawMode.unwrap()(mode) }
  }

  /// Clears `bitmap`, filling with the given `bgcolor`.
  pub fn clear_bitmap<'a, C>(&self, bitmap: &LCDBitmap, bgcolor: C)
  where
    C: Into<LCDColor<'a>>,
  {
    unsafe {
      self.state.cgraphics.clearBitmap.unwrap()(bitmap.bitmap_ptr, bgcolor.into().to_c_color());
    }
  }

  /// Returns a new `LCDBitmap` that is an exact copy of `bitmap`.
  pub fn copy_bitmap(&self, bitmap: &LCDBitmap) -> LCDBitmap {
    LCDBitmap {
      bitmap_ptr: unsafe { self.state.cgraphics.copyBitmap.unwrap()(bitmap.bitmap_ptr) },
      state: self.state,
    }
  }

  // TODO: checkMaskCollision

  /// Draws the bitmap with its upper-left corner at location (`x`, `y`), using the given `flip`
  /// orientation.
  pub fn draw_bitmap(&self, bitmap: &LCDBitmap, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe { self.state.cgraphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip) }
  }

  /// Draws the bitmap scaled to `xscale` and `yscale` with its upper-left corner at location
  /// (`x`, `y`). Note that flip is not available when drawing scaled bitmaps but negative scale
  /// values will achieve the same effect.
  pub fn draw_scaled_bitmap(&self, bitmap: &LCDBitmap, x: i32, y: i32, xscale: f32, yscale: f32) {
    unsafe {
      self.state.cgraphics.drawScaledBitmap.unwrap()(bitmap.bitmap_ptr, x, y, xscale, yscale)
    }
  }

  /// Draws the bitmap scaled to `xscale` and `yscale` then rotated by `degrees` with its center
  /// as given by proportions `centerx` and `centery` at (`x`, `y`); that is: if `centerx` and
  /// `centery` are both 0.5 the center of the image is at (`x`, `y`), if `centerx` and `centery`
  /// are both 0 the top left corner of the image (before rotation) is at (`x`, `y`), etc.
  pub fn draw_rotated_bitmap(
    &self,
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

  /// Get `LCDBitmapData` containing info about `bitmap` such as `width`, `height`, and raw pixel
  /// data.
  pub fn get_bitmap_data<'a>(&self, bitmap: &'a LCDBitmap) -> LCDBitmapData<'a> {
    // This exists to match the API.
    bitmap.data()
  }

  pub fn load_bitmap<S>(&self, path: S) -> Result<LCDBitmap, Error>
  where
    S: AsRef<str>,
  {
    use crate::null_terminated::ToNullTerminatedString;
    let path = path.as_ref().to_null_terminated_utf8().as_ptr();
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
            return Err(Error(String::from("LoadBitmap: ") + &out_err));
          }
        }
      }

      return Err(Error(String::from("LoadBitmap: unknown error")));
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
    C: Into<LCDColor<'a>>,
  {
    // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
    let bitmap_ptr = unsafe {
      self.state.cgraphics.newBitmap.unwrap()(width, height, bg_color.into().to_c_color())
    };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }

  /// Draws the bitmap with its upper-left corner at location (`x`, `y`) tiled inside a
  /// `width` by `height` rectangle.
  pub fn tile_bitmap(
    &self,
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

  /// Returns a new, rotated and scaled LCDBitmap based on the given `bitmap`.
  pub fn rotated_bitmap(
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

  pub fn draw_text<S>(&self, text: S, encoding: PDStringEncoding, x: i32, y: i32)
  where
    S: AsRef<str>,
  {
    use crate::null_terminated::ToNullTerminatedString;
    let null_term = text.as_ref().to_null_terminated_utf8();
    let ptr = null_term.as_ptr() as *const c_void;
    let len = null_term.len() as u64;
    unsafe { self.state.cgraphics.drawText.unwrap()(ptr, len, encoding, x, y) }; // TODO: Return the int from Playdate?
  }

  /// Draws the current FPS on the screen at the given (`x`, `y`) coordinates.
  pub fn draw_fps(&self, x: i32, y: i32) {
    // This function is part of Playdate CSystem, not CGraphics, but it's a function that draws
    // something to the screen, so its behaviour is more clear when part of the Graphics type.
    unsafe { self.state.csystem.drawFPS.unwrap()(x, y) }
  }
}
