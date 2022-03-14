use core::ffi::c_void;
use core::marker::PhantomData;

use crate::capi_state::CApiState;
use crate::ctypes::*;

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

impl LCDBitmap {
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

pub struct LCDBitmapData<'a> {
  pub width: i32,
  pub height: i32,
  pub rowbytes: i32,
  // TODO: is hasmask logically a boolean?
  pub hasmask: i32,
  // TODO: direct access into the bitmap, so does not need to be freed?
  pub data: *mut u8,
  // Share lifetime of LCDBitmap that generated this.
  phantom: PhantomData<&'a ()>,
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

  pub fn set_draw_mode(&self, mode: LCDBitmapDrawMode) {
    unsafe { self.state.graphics.setDrawMode.unwrap()(mode) }
  }

  // FIXME: for some reason, patterns don't appear to work here, but do work with a C example.
  pub fn new_bitmap(&self, width: i32, height: i32, bg_color: LCDColor) -> LCDBitmap {
    let bitmap_ptr =
      unsafe { self.state.graphics.newBitmap.unwrap()(width, height, bg_color.as_c_color()) };
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
    use crate::null_terminated::ToNullTerminated;
    let null_term = text.as_ref().to_null_terminated();
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
