use core::ffi::c_void;

use crate::api::Error;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::String;

const PATTERN_SIZE: usize = 16;

#[derive(Debug)]
#[repr(transparent)]
pub struct LCDPattern(CLCDPattern);
impl LCDPattern {
  pub fn new(arr: [u8; PATTERN_SIZE]) -> LCDPattern {
    LCDPattern(arr)
  }
  pub fn from_bitmap(bitmap: &LCDBitmapRef, x: i32, y: i32) -> LCDPattern {
    let mut arr = [0; PATTERN_SIZE];

    let mut c_color: CLCDColor = 0;
    unsafe {
      // The setColorToPattern function wants a `*mut CLCDBitmap`, but it only reads from the bitmap
      // to make a pattern, so we can cast to that from a shared reference to the bitmap.
      let ptr = bitmap.get_bitmap_ptr() as *mut CLCDBitmap;
      bitmap.state.cgraphics.setColorToPattern.unwrap()(&mut c_color, ptr, x, y);
      core::ptr::copy_nonoverlapping(c_color as *const u8, arr.as_mut_ptr(), PATTERN_SIZE)
    }

    LCDPattern(arr)
  }
}

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
      LCDColor::Pattern(pattern) => pattern.0.as_ptr() as usize,
    }
  }
}

/// A bitmap image.
///
/// The bitmap can be cloned which will make a clone of the pixels as well. The bitmap's pixels data
/// is freed when the bitmap is dropped.
///
/// An `LCDBitmap` is borrowed as an `&LCDBitmapRef` and all methods of that type are available for
/// `LCDBitmap as well.
#[derive(Debug)]
pub struct LCDBitmap {
  /// While LCDBitmapRef is a non-owning pointer, the LCDBitmap will act as the owner of the bitmap
  /// found within.
  owned: LCDBitmapRef,
}
impl LCDBitmap {
  /// Construct an LCDBitmap from an owning pointer.
  fn from_owned_ptr(bitmap_ptr: *mut CLCDBitmap, state: &'static CApiState) -> Self {
    LCDBitmap {
      owned: LCDBitmapRef::from_ptr(bitmap_ptr, state),
    }
  }
}

impl Clone for LCDBitmap {
  fn clone(&self) -> Self {
    LCDBitmap::from_owned_ptr(
      unsafe { self.owned.state.cgraphics.copyBitmap.unwrap()(self.owned.bitmap_ptr) },
      self.owned.state,
    )
  }
}

impl Drop for LCDBitmap {
  fn drop(&mut self) {
    unsafe {
      self.owned.state.cgraphics.freeBitmap.unwrap()(self.owned.bitmap_ptr);
    }
  }
}

/// A reference to an `LCDBitmap`, which has a lifetime tied to a different `LCDBitmap` (or
/// `LCDBitmapRef`) with a lifetime `'a`.
#[derive(Debug)]
pub struct SharedLCDBitmapRef<'a> {
  bref: LCDBitmapRef,
  _marker: core::marker::PhantomData<&'a LCDBitmap>,
}

impl SharedLCDBitmapRef<'_> {
  /// Construct a SharedLCDBitmapRef from a non-owning pointer.
  ///
  /// Requires being told the lifetime of the LCDBitmap this is making a reference to.
  fn from_ptr<'a>(
    bitmap_ptr: *mut CLCDBitmap,
    state: &'static CApiState,
  ) -> SharedLCDBitmapRef<'a> {
    SharedLCDBitmapRef {
      bref: LCDBitmapRef::from_ptr(bitmap_ptr, state),
      _marker: core::marker::PhantomData,
    }
  }
}

/// A borrow of an LCDBitmap (or SharedLCDBitmap) is held as this type.
///
/// LCDBitmapRef exposes most of the method of an LCDBitmap, allowing them to be used on an owned or
/// borrowed bitmap.
///
/// Intentionally not `Copy` as `LCDBitmapRef` can only be referred to as a reference.
#[derive(Debug)]
pub struct LCDBitmapRef {
  bitmap_ptr: *mut CLCDBitmap,
  state: &'static CApiState,
}

impl LCDBitmapRef {
  /// Construct an LCDBitmapRef from a non-owning pointer.
  fn from_ptr(bitmap_ptr: *mut CLCDBitmap, state: &'static CApiState) -> Self {
    LCDBitmapRef { bitmap_ptr, state }
  }

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

  /// Returns the bitmap's metadata such as its width and height.
  pub fn data(&self) -> LCDBitmapData {
    let (data, _) = self.data_and_pixels_ptr();
    data
  }

  /// Gives read acccess to the pixels of the bitmap as an array of bytes.
  ///
  /// Each byte represents 8 pixels, where each pixel is a bit. The highest bit is the leftmost
  /// pixel, and lowest bit is the rightmost.
  pub fn as_bytes(&self) -> &[u8] {
    let (data, pixels) = self.data_and_pixels_ptr();
    unsafe { core::slice::from_raw_parts(pixels, (data.rowbytes * data.height) as usize) }
  }
  /// Gives read-write acccess to the pixels of the bitmap as an array of bytes.
  ///
  /// Each byte represents 8 pixels, where each pixel is a bit. The highest bit is the leftmost
  /// pixel, and lowest bit is the rightmost.
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

  /// Sets a mask image for the given bitmap. The set mask must be the same size as the target
  /// bitmap.
  ///
  /// The mask bitmap is copied, so no reference is held to it.
  pub fn set_mask_bitmap(&mut self, mask: &LCDBitmapRef) -> Result<(), Error> {
    // Playdate makes a copy of the mask bitmap.
    let result =
      unsafe { self.state.cgraphics.setBitmapMask.unwrap()(self.bitmap_ptr, mask.bitmap_ptr) };
    match result {
      1 => Ok(()),
      0 => Err("failed to set mask bitmap, dimensions to not match".into()),
      _ => panic!("unknown error result from setBitmapMask"),
    }
  }

  /// The mask bitmap attached to this bitmap.
  ///
  /// Returns the mask bitmap, if one has been attached with `set_mask_bitmap()`, or None.
  pub fn mask_bitmap(&self) -> Option<SharedLCDBitmapRef> {
    let mask = unsafe {
      // Playdate owns the mask bitmap, and reference a pointer to it. Playdate would free the mask
      // presumably when `self` is freed.
      self.state.cgraphics.getBitmapMask.unwrap()(self.bitmap_ptr)
    };
    if !mask.is_null() {
      Some(SharedLCDBitmapRef::from_ptr(mask, self.state))
    } else {
      None
    }
  }

  pub(crate) unsafe fn get_bitmap_ptr(&self) -> *const CLCDBitmap {
    self.bitmap_ptr
  }
}

impl core::ops::Deref for LCDBitmap {
  type Target = LCDBitmapRef;

  fn deref(&self) -> &Self::Target {
    &self.owned
  }
}
impl core::ops::DerefMut for LCDBitmap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.owned
  }
}

impl core::borrow::Borrow<LCDBitmapRef> for LCDBitmap {
  fn borrow(&self) -> &LCDBitmapRef {
    self // This calls Deref.
  }
}
impl core::borrow::BorrowMut<LCDBitmapRef> for LCDBitmap {
  fn borrow_mut(&mut self) -> &mut LCDBitmapRef {
    self // This calls DerefMut.
  }
}

impl alloc::borrow::ToOwned for LCDBitmapRef {
  type Owned = LCDBitmap;

  fn to_owned(&self) -> Self::Owned {
    LCDBitmap::from_owned_ptr(
      unsafe { self.state.cgraphics.copyBitmap.unwrap()(self.bitmap_ptr) },
      self.state,
    )
  }
}

impl core::ops::Deref for SharedLCDBitmapRef<'_> {
  type Target = LCDBitmapRef;

  fn deref(&self) -> &Self::Target {
    &self.bref
  }
}
impl core::ops::DerefMut for SharedLCDBitmapRef<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.bref
  }
}

impl core::borrow::Borrow<LCDBitmapRef> for SharedLCDBitmapRef<'_> {
  fn borrow(&self) -> &LCDBitmapRef {
    self // This calls Deref.
  }
}
impl core::borrow::BorrowMut<LCDBitmapRef> for SharedLCDBitmapRef<'_> {
  fn borrow_mut(&mut self) -> &mut LCDBitmapRef {
    self // This calls DerefMut.
  }
}

impl AsRef<LCDBitmapRef> for LCDBitmap {
  fn as_ref(&self) -> &LCDBitmapRef {
    self // This calls Deref.
  }
}
impl AsMut<LCDBitmapRef> for LCDBitmap {
  fn as_mut(&mut self) -> &mut LCDBitmapRef {
    self // This calls DerefMut.
  }
}
impl AsRef<LCDBitmapRef> for SharedLCDBitmapRef<'_> {
  fn as_ref(&self) -> &LCDBitmapRef {
    self // This calls Deref.
  }
}
impl AsMut<LCDBitmapRef> for SharedLCDBitmapRef<'_> {
  fn as_mut(&mut self) -> &mut LCDBitmapRef {
    self // This calls DerefMut.
  }
}
impl AsRef<LCDBitmapRef> for LCDBitmapRef {
  fn as_ref(&self) -> &LCDBitmapRef {
    self
  }
}
impl AsMut<LCDBitmapRef> for LCDBitmapRef {
  fn as_mut(&mut self) -> &mut LCDBitmapRef {
    self
  }
}

/// Metadata for an `LCDBitmap`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LCDBitmapData {
  width: i32,
  height: i32,
  rowbytes: i32,
  hasmask: i32,
}
impl LCDBitmapData {
  /// The number of pixels (or, columns) per row of the bitmap.
  ///
  /// Each pixel is a single bit, and there may be more bytes (as determined by `row_bytes()`) in a
  /// row than required to hold all the pixels.
  pub fn width(&self) -> i32 {
    self.width
  }
  /// The number of rows in the bitmap.
  pub fn height(&self) -> i32 {
    self.height
  }
  /// The number of bytes per row of the bitmap.
  pub fn row_bytes(&self) -> i32 {
    self.rowbytes
  }
  /// Whether the bitmap has a mask attached, via `set_mask_bitmap()`.
  pub fn has_mask(&self) -> bool {
    self.hasmask != 0
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

  /// Returns the debug framebuffer as a bitmap.
  ///
  /// Only valid in the simulator, so not compiled for device builds.
  #[cfg(not(all(target_arch = "arm", target_os = "none")))]
  pub fn debug_frame_bitmap(&self) -> SharedLCDBitmapRef<'static> {
    let bitmap_ptr = unsafe { self.state.cgraphics.getDebugBitmap.unwrap()() };
    assert!(!bitmap_ptr.is_null());
    SharedLCDBitmapRef::from_ptr(bitmap_ptr, self.state)
  }

  /// Returns a copy of the contents of the display front buffer.
  ///
  /// The Playdate device is double-buffered, and this returns the currently displayed frame.
  pub fn display_frame_bitmap(&self) -> LCDBitmap {
    let bitmap_ptr = unsafe { self.state.cgraphics.getDisplayBufferBitmap.unwrap()() };
    use alloc::borrow::ToOwned;
    LCDBitmapRef::from_ptr(bitmap_ptr, self.state).to_owned()
  }

  /// Returns a copy the contents of the working frame buffer as a bitmap.
  ///
  /// The Playdate device is double-buffered, and this returns the buffer that will be displayed
  /// next frame.
  pub fn working_frame_bitmap(&self) -> LCDBitmap {
    let bitmap_ptr = unsafe { self.state.cgraphics.copyFrameBufferBitmap.unwrap()() };
    LCDBitmap::from_owned_ptr(bitmap_ptr, self.state)
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
  pub fn draw_bitmap(&mut self, bitmap: &LCDBitmapRef, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe { self.state.cgraphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip) }
  }

  /// Draws the bitmap to the screen, scaled by `xscale` and `yscale`.
  ///
  /// /// The bitmap's upper-left corner is positioned at location (`x`, `y`). Note that flip is not
  /// available when drawing scaled bitmaps but negative scale values will achieve the same effect.
  pub fn draw_scaled_bitmap(
    &mut self,
    bitmap: &LCDBitmapRef,
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
    bitmap: &LCDBitmapRef,
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
    bitmap: &LCDBitmapRef,
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

    Ok(LCDBitmap::from_owned_ptr(bitmap_ptr, self.state))
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
    LCDBitmap::from_owned_ptr(bitmap_ptr, self.state)
  }

  /// Returns a new, rotated and scaled LCDBitmap based on the given `bitmap`.
  pub fn new_rotated_bitmap(
    &self,
    bitmap: &LCDBitmapRef,
    rotation: f32,
    xscale: f32,
    yscale: f32,
  ) -> LCDBitmap {
    // This function could grow the bitmap by rotating and so it (conveniently?) also returns the
    // alloced size of the new bitmap.  You can get this off the bitmap data more or less if needed.
    let mut _alloced_size: i32 = 0;
    let bitmap_ptr = unsafe {
      self.state.cgraphics.rotatedBitmap.unwrap()(
        bitmap.bitmap_ptr,
        rotation,
        xscale,
        yscale,
        &mut _alloced_size,
      )
    };
    LCDBitmap::from_owned_ptr(bitmap_ptr, self.state)
  }

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
