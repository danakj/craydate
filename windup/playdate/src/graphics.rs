use core::ffi::c_void;

use crate::api::Error;
use crate::bitmap::{Bitmap, BitmapRef, SharedBitmapRef};
use crate::capi_state::{CApiState, ContextStackId};
use crate::color::Color;
use crate::ctypes::*;
use crate::font::Font;
use crate::format;
use crate::null_terminated::ToNullTerminatedString;

pub struct BitmapCollider<'a> {
  pub bitmap: &'a BitmapRef,
  pub flipped: BitmapFlip,
  pub x: i32,
  pub y: i32,
}

#[derive(Debug)]
pub struct Graphics {
  pub(crate) state: &'static CApiState,
}
impl Graphics {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Graphics { state }
  }

  pub fn bitmaps_collide(
    &self,
    a: BitmapCollider,
    b: BitmapCollider,
    in_rect: euclid::default::Rect<i32>,
  ) -> bool {
    unsafe {
      // checkMaskCollision expects `*mut CLCDBitmap` but it only reads from the bitmaps to check
      // for collision, so we can cast from a shared reference on Bitmap to a mut pointer.
      self.state.cgraphics.checkMaskCollision.unwrap()(
        a.bitmap.as_bitmap_ptr(),
        a.x,
        a.y,
        a.flipped,
        b.bitmap.as_bitmap_ptr(),
        b.x,
        b.y,
        b.flipped,
        playdate_rect_from_euclid(in_rect),
      ) != 0
    }
  }

  /// Clears the entire display, filling it with `color`.
  pub fn clear<'a, C>(&mut self, color: C)
  where
    Color<'a>: From<C>,
  {
    unsafe {
      self.state.cgraphics.clear.unwrap()(Color::<'a>::from(color).to_c_color());
    }
  }

  /// Sets the background color shown when the display is offset or for clearing dirty areas
  /// in the sprite system.
  pub fn set_background_color(&mut self, color: SolidColor) {
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
  pub fn debug_frame_bitmap(&self) -> SharedBitmapRef<'static> {
    let bitmap_ptr = unsafe { self.state.cgraphics.getDebugBitmap.unwrap()() };
    assert!(!bitmap_ptr.is_null());
    SharedBitmapRef::from_ptr(bitmap_ptr, self.state)
  }

  /// Returns a copy of the contents of the display front buffer.
  ///
  /// The Playdate device is double-buffered, and this returns the currently displayed frame.
  pub fn display_frame_bitmap(&self) -> Bitmap {
    let bitmap_ptr = unsafe { self.state.cgraphics.getDisplayBufferBitmap.unwrap()() };
    use alloc::borrow::ToOwned;
    BitmapRef::from_ptr(bitmap_ptr, self.state).to_owned()
  }

  /// Returns a copy the contents of the working frame buffer as a bitmap.
  ///
  /// The Playdate device is double-buffered, and this returns the buffer that will be displayed
  /// next frame.
  pub fn working_frame_bitmap(&self) -> Bitmap {
    let bitmap_ptr = unsafe { self.state.cgraphics.copyFrameBufferBitmap.unwrap()() };
    Bitmap::from_owned_ptr(bitmap_ptr, self.state)
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

  /// Push a new drawing context that targets the display framebuffer.
  ///
  /// Drawing functions use a context stack to select the drawing target, for setting a stencil,
  /// changing the draw mode, etc. The stack is unwound at the beginning of each update cycle, with
  /// drawing restored to target the display framebuffer.
  pub fn push_context(&mut self) {
    self.state.stack.borrow_mut().push_framebuffer(self.state)
  }
  /// Push a drawing context that targets a bitmap.
  ///
  /// Drawing functions use a context stack to select the drawing target, for setting a stencil,
  /// changing the draw mode, etc. The stack is unwound at the beginning of each update cycle, with
  /// drawing restored to target the display framebuffer.
  ///
  /// When the bitmap's drawing is popped, either by calling pop_context() or at the end of the
  /// frame, it will be kept alive as long as the ContextStackId returned here (or a clone of it) is
  /// kept alive.
  pub fn push_context_bitmap(&mut self, bitmap: Bitmap) -> ContextStackId {
    self.state.stack.borrow_mut().push_bitmap(self.state, bitmap)
  }
  /// Pop the top (most recently pushed, and not yet popped) drawing context from the stack.
  ///
  /// Drawing functions use a context stack to select the drawing target, for setting a stencil,
  /// changing the draw mode, etc. The stack is unwound at the beginning of each update cycle, with
  /// drawing restored to target the display framebuffer.
  ///
  /// The returned ContextStackId, if present, can be used to get back the Bitmap that was drawn
  /// into for the popped drawing context. A ContextStackId is not returned if the popped drawing
  /// context was drawing into the display framebuffer.
  pub fn pop_context(&mut self) -> Option<ContextStackId> {
    self.state.stack.borrow_mut().pop(self.state)
  }
  /// Retrieve an Bitmap that was pushed into a drawing context with push_context_bitmap() and
  /// since popped off the stack, either with pop_context() or at the end of the frame.
  pub fn take_popped_context_bitmap(&mut self, id: ContextStackId) -> Option<Bitmap> {
    self.state.stack.borrow_mut().take_bitmap(id)
  }

  /// Sets the stencil used for drawing.
  ///
  /// If the image is smaller than full screen, its width should be a multiple of 32 pixels.
  /// Stencils smaller than full screen will be tiled.
  ///
  /// The bitmap will remain the stencil as long as the FramebufferStencilBitmap is not dropped, or another
  /// call to set_stencil() is made.
  pub fn set_stencil<'a>(&mut self, bitmap: &'a BitmapRef) -> FramebufferStencilBitmap<'a> {
    let gen = self.state.stencil_generation.get() + 1;
    self.state.stencil_generation.set(gen);
    unsafe { self.state.cgraphics.setStencil.unwrap()(bitmap.as_bitmap_ptr()) }
    FramebufferStencilBitmap {
      state: self.state,
      // Track the generation number so as to only unset the stencil on drop if set_stencil() wasn't
      // called again since.
      generation: gen,
      bitmap,
    }
  }

  /// Sets the font used for drawing.
  ///
  /// The font will remain active for drawing as long as the ActiveFont is not dropped, or another
  /// call to set_font() is made.
  pub fn set_font<'a>(&mut self, font: &'a Font) -> ActiveFont<'a> {
    let gen = self.state.font_generation.get() + 1;
    self.state.font_generation.set(gen);
    unsafe { self.state.cgraphics.setFont.unwrap()(font.as_ptr()) }
    ActiveFont {
      state: self.state,
      // Track the generation number so as to only unset the font on drop if set_font() wasn't
      // called again since.
      generation: gen,
      font,
    }
  }

  /// Sets the current clip rect, using world coordinates—​that is, the given rectangle will be
  /// translated by the current drawing offset.
  ///
  /// The clip rect is cleared at the beginning of each frame.
  pub fn set_clip_rect(&mut self, rect: euclid::default::Rect<i32>) {
    unsafe {
      self.state.cgraphics.setClipRect.unwrap()(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
      )
    }
  }
  /// Sets the current clip rect in screen coordinates.
  ///
  /// The clip rect is cleared at the beginning of each frame.
  pub fn set_screen_clip_rect(&mut self, rect: euclid::default::Rect<i32>) {
    unsafe {
      self.state.cgraphics.setScreenClipRect.unwrap()(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
      )
    }
  }

  // TODO: all the graphics->video functions

  /// Sets the mode used for drawing bitmaps. Note that text drawing uses bitmaps, so this
  /// affects how fonts are displayed as well.
  pub fn set_draw_mode(&mut self, mode: BitmapDrawMode) {
    unsafe { self.state.cgraphics.setDrawMode.unwrap()(mode) }
  }

  /// Draws the bitmap to the screen.
  ///
  /// The bitmap's upper-left corner is positioned at location (`x`, `y`), and the contents have
  /// the `flip` orientation applied.
  pub fn draw_bitmap(&mut self, bitmap: &BitmapRef, x: i32, y: i32, flip: BitmapFlip) {
    unsafe { self.state.cgraphics.drawBitmap.unwrap()(bitmap.as_bitmap_ptr(), x, y, flip) }
  }

  /// Draws the bitmap to the screen, scaled by `xscale` and `yscale`.
  ///
  /// /// The bitmap's upper-left corner is positioned at location (`x`, `y`). Note that flip is not
  /// available when drawing scaled bitmaps but negative scale values will achieve the same effect.
  pub fn draw_scaled_bitmap(
    &mut self,
    bitmap: &BitmapRef,
    x: i32,
    y: i32,
    xscale: f32,
    yscale: f32,
  ) {
    unsafe {
      self.state.cgraphics.drawScaledBitmap.unwrap()(bitmap.as_bitmap_ptr(), x, y, xscale, yscale)
    }
  }

  /// Draws the bitmap to the screen, scaled by `xscale` and `yscale` then rotated by `degrees` with
  /// its center as given by proportions `centerx` and `centery` at (`x`, `y`); that is: if
  /// `centerx` and `centery` are both 0.5 the center of the image is at (`x`, `y`), if `centerx`
  /// and `centery` are both 0 the top left corner of the image (before rotation) is at (`x`, `y`),
  /// etc.
  pub fn draw_rotated_bitmap(
    &mut self,
    bitmap: &BitmapRef,
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
        bitmap.as_bitmap_ptr(),
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
    bitmap: &BitmapRef,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flip: BitmapFlip,
  ) {
    unsafe {
      self.state.cgraphics.tileBitmap.unwrap()(bitmap.as_bitmap_ptr(), x, y, width, height, flip)
    }
  }

  /// Returns the Font object for the font file at `path`.
  pub fn load_font(&self, path: &str) -> Result<Font, Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    let font_ptr = unsafe {
      self.state.cgraphics.loadFont.unwrap()(path.to_null_terminated_utf8().as_ptr(), &mut out_err)
    };

    if !out_err.is_null() {
      let result = unsafe { crate::null_terminated::parse_null_terminated_utf8(out_err) };
      match result {
        // A valid error string.
        Ok(err) => Err(format!("load_font: {}", err).into()),
        // An invalid error string.
        Err(err) => Err(format!("load_font: unknown error ({})", err).into()),
      }
    } else {
      assert!(!font_ptr.is_null());
      Ok(Font::from_ptr(font_ptr, self.state))
    }
  }

  pub fn load_bitmap(&self, path: &str) -> Result<Bitmap, Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    let bitmap_ptr = unsafe {
      self.state.cgraphics.loadBitmap.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        &mut out_err,
      )
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
      Ok(Bitmap::from_owned_ptr(bitmap_ptr, self.state))
    }
  }

  /// Loads the image at `path` into the previously allocated `bitmap`.
  pub fn load_into_bitmap(&self, path: &str, bitmap: &mut BitmapRef) -> Result<(), Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    unsafe {
      self.state.cgraphics.loadIntoBitmap.unwrap()(
        path.to_null_terminated_utf8().as_ptr(),
        bitmap.as_bitmap_mut_ptr(),
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

  /// Allocates and returns a new `width` by `height` `Bitmap` filled with `bg_color`.
  pub fn new_bitmap<'a, C>(&self, width: i32, height: i32, bg_color: C) -> Bitmap
  where
    Color<'a>: From<C>,
  {
    // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
    let bitmap_ptr = unsafe {
      self.state.cgraphics.newBitmap.unwrap()(
        width,
        height,
        Color::<'a>::from(bg_color).to_c_color(),
      )
    };
    Bitmap::from_owned_ptr(bitmap_ptr, self.state)
  }

  /// Returns a new, rotated and scaled Bitmap based on the given `bitmap`.
  pub fn new_rotated_bitmap(
    &self,
    bitmap: &BitmapRef,
    rotation: f32,
    xscale: f32,
    yscale: f32,
  ) -> Bitmap {
    // This function could grow the bitmap by rotating and so it (conveniently?) also returns the
    // alloced size of the new bitmap.  You can get this off the bitmap data more or less if needed.
    let mut _alloced_size: i32 = 0;
    let bitmap_ptr = unsafe {
      self.state.cgraphics.rotatedBitmap.unwrap()(
        bitmap.as_bitmap_ptr(),
        rotation,
        xscale,
        yscale,
        &mut _alloced_size,
      )
    };
    Bitmap::from_owned_ptr(bitmap_ptr, self.state)
  }

  // TODO: getTableBitmap
  // TODO: loadBitmapTable
  // TODO: loadIntoBitmapTable
  // TODO: newBitmapTable

  pub fn draw_text(&mut self, text: &str, encoding: StringEncoding, x: i32, y: i32) {
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

  /// Draws an ellipse inside the rectangle of width `line_width` (inset from the rectangle bounds).
  ///
  /// If `start_deg != end_deg`, this draws an arc between the given angles. Angles are given in
  /// degrees, clockwise from due north.
  pub fn draw_elipse<'a>(
    &mut self,
    rect: euclid::default::Rect<i32>,
    line_width: i32,
    start_deg: f32,
    end_deg: f32,
    color: Color<'a>,
  ) {
    unsafe {
      self.state.cgraphics.drawEllipse.unwrap()(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
        line_width,
        start_deg,
        end_deg,
        color.to_c_color(),
      )
    }
  }
  /// Fills an ellipse inside the rectangle.
  ///
  /// If `start_deg != end_deg`, this draws an arc between the given angles. Angles are given in
  /// degrees, clockwise from due north.
  pub fn fill_elipse<'a>(
    &mut self,
    rect: euclid::default::Rect<i32>,
    start_deg: f32,
    end_deg: f32,
    color: Color<'a>,
  ) {
    unsafe {
      self.state.cgraphics.fillEllipse.unwrap()(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
        start_deg,
        end_deg,
        color.to_c_color(),
      )
    }
  }
  /// Draws a line from `p1` to `p2` with a stroke width of `width`.
  pub fn draw_line<'a>(
    &mut self,
    p1: euclid::default::Point2D<i32>,
    p2: euclid::default::Point2D<i32>,
    line_width: i32,
    color: Color<'a>,
  ) {
    unsafe {
      self.state.cgraphics.drawLine.unwrap()(p1.x, p1.y, p2.x, p2.y, line_width, color.to_c_color())
    }
  }
  /// Draws a `rect`.
  pub fn draw_rect<'a>(&mut self, r: euclid::default::Rect<i32>, color: Color<'a>) {
    unsafe {
      self.state.cgraphics.drawRect.unwrap()(
        r.origin.x,
        r.origin.y,
        r.size.width,
        r.size.height,
        color.to_c_color(),
      )
    }
  }
  /// Draws a filled `rect`.
  pub fn fill_rect<'a>(&mut self, r: euclid::default::Rect<i32>, color: Color<'a>) {
    unsafe {
      self.state.cgraphics.fillRect.unwrap()(
        r.origin.x,
        r.origin.y,
        r.size.width,
        r.size.height,
        color.to_c_color(),
      )
    }
  }
  /// Draws a filled triangle with points at `p1`, `p2`, and `p3`.
  pub fn fill_triangle<'a>(
    &mut self,
    p1: euclid::default::Point2D<i32>,
    p2: euclid::default::Point2D<i32>,
    p3: euclid::default::Point2D<i32>,
    color: Color<'a>,
  ) {
    unsafe {
      self.state.cgraphics.fillTriangle.unwrap()(
        p1.x,
        p1.y,
        p2.x,
        p2.y,
        p3.x,
        p3.y,
        color.to_c_color(),
      )
    }
  }
  /// Fills the polygon with vertices at the given coordinates (an array of points) using the given color and fill, or winding, rule.
  ///
  /// See https://en.wikipedia.org/wiki/Nonzero-rule for an explanation of the winding rule.
  pub fn fill_polygon<'a>(
    &mut self,
    points: &[euclid::default::Point2D<i32>],
    color: Color<'a>,
    fill_rule: PolygonFillRule,
  ) {
    // Point2D is a #[repr(C)] struct of x, y. It's alignment will be the same as i32, so an
    // array of Point2D can be treated as an array of i32 with x and y alternating.
    let raw_points = points.as_ptr() as *mut i32;
    unsafe {
      self.state.cgraphics.fillPolygon.unwrap()(
        points.len() as i32,
        raw_points,
        color.to_c_color(),
        fill_rule,
      )
    }
  }
}

fn playdate_rect_from_euclid(e: euclid::default::Rect<i32>) -> CLCDRect {
  CLCDRect {
    left: e.origin.x,
    top: e.origin.y,
    right: e.origin.x + e.size.width - 1,
    bottom: e.origin.y + e.size.height - 1,
  }
}

/// A sentinel that marks a bitmap acting as the stencil for drawing. Destroying this object will
/// unset the bitmap as the stencil.
pub struct FramebufferStencilBitmap<'a> {
  state: &'static CApiState,
  generation: usize,
  bitmap: &'a BitmapRef,
}
impl<'a> FramebufferStencilBitmap<'a> {
  pub fn bitmap(&self) -> &'a BitmapRef {
    self.bitmap
  }
}
impl Drop for FramebufferStencilBitmap<'_> {
  fn drop(&mut self) {
    if self.generation == self.state.stencil_generation.get() {
      unsafe { self.state.cgraphics.setStencil.unwrap()(core::ptr::null_mut()) }
    }
  }
}

/// A sentinel that marks a font as the currently active font. Destroying this object will
/// unset the font as current.
pub struct ActiveFont<'a> {
  state: &'static CApiState,
  generation: usize,
  font: &'a Font,
}
impl<'a> ActiveFont<'a> {
  pub fn font(&self) -> &'a Font {
    self.font
  }
}
impl Drop for ActiveFont<'_> {
  fn drop(&mut self) {
    if self.generation == self.state.font_generation.get() {
      unsafe { self.state.cgraphics.setFont.unwrap()(core::ptr::null_mut()) }
    }
  }
}
