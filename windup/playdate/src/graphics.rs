use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::{CStr, CString};

#[derive(Debug)]
pub enum LCDColor<'a> {
  Solid(LCDSolidColor),
  Pattern(&'a LCDPattern),
}

impl<'a> LCDColor<'a> {
  pub unsafe fn as_c_color(&self) -> usize {
    // SAFETY: the returned usize for patterns is technically a raw pointer to the LCDPattern
    // array itself.  It must be passed to Playdate before the LCDColor is dead or moved.
    // Also, yes really, LCDColor can be both an enum and a pointer.
    match self {
      LCDColor::Solid(color) => color.0 as usize,
      LCDColor::Pattern(&color) => color.as_ptr() as usize,
    }
  }
}

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

#[derive(Debug)]
pub struct Graphics {
  pub(crate) state: &'static CApiState,
}
impl Graphics {
  pub fn clear<'a>(&self, color: LCDColor<'a>) {
    unsafe {
      self.state.graphics.clear.unwrap()(color.as_c_color());
    }
  }

  // NOTE: it appears in practice that new_bitmap's bg_color parameter is only
  // interpreted as an LCDSolidColor and not as an LCDColor/LCDPattern.
  pub fn new_bitmap(&self, width: i32, height: i32, bg_color: LCDSolidColor) -> LCDBitmap {
    let bg_color = LCDColor::Solid(bg_color);
    let bitmap_ptr =
      unsafe { self.state.graphics.newBitmap.unwrap()(width, height, bg_color.as_c_color()) };
    LCDBitmap {
      bitmap_ptr,
      state: self.state,
    }
  }

  pub fn draw_bitmap(&self, bitmap: &LCDBitmap, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe {
      self.state.graphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip);
    }
  }

  pub fn draw_text(&self, text: &CStr, encoding: PDStringEncoding, x: i32, y: i32) {
    let len = text.to_bytes().len() as u64;
    unsafe {
      let text = text.as_ptr() as *const core::ffi::c_void;
      self.state.graphics.drawText.unwrap()(text, len, encoding, x, y);
    }
  }
}
