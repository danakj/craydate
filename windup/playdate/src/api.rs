use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::executor::Executor;
use crate::CStr;

#[derive(Debug)]
pub struct Api {
  pub system: System,
  pub graphics: Graphics,
}
impl Api {
  pub(crate) fn new(state: &'static CApiState) -> Api {
    Api {
      system: System { state },
      graphics: Graphics { state },
    }
  }
}

#[derive(Debug)]
pub struct System {
  pub(crate) state: &'static CApiState,
}
impl System {
  /// A watcher that lets you `await` for the next frame update from the Playdate device.
  pub fn frame_watcher(&self) -> FrameWatcher {
    FrameWatcher {
      state: self.state,
    }
  }

  pub fn log(&self, s: &CStr) {
    unsafe { self.state.system.logToConsole.unwrap()(s.as_ptr()) };
  }
}

#[derive(Debug)]
pub struct FrameWatcher {
  state: &'static CApiState,
}
impl FrameWatcher {
  /// Runs until the next frame from the Playdate device, then returns the frame number.
  /// 
  /// This function returns after the Playdate device calls the "update callback" to signify that
  /// the game should perform updates for the next frame to be displayed.
  pub async fn next(&self) -> u64 {
    self.next_impl().await
  }
  fn next_impl(&self) -> FrameWatcherFuture {
    FrameWatcherFuture {
      state: self.state,
      seen_frame: self.state.frame_number.get(),
    }
  }
}

/// A future for which poll() waits for the next update, then returns Complete.
struct FrameWatcherFuture {
  state: &'static CApiState,
  seen_frame: u64,
}

impl Future for FrameWatcherFuture {
  type Output = u64;

  fn poll(self: Pin<&mut Self>, ctxt: &mut Context<'_>) -> Poll<u64> {
    let frame = self.state.frame_number.get();

    if frame > self.seen_frame {
      Poll::Ready(frame)
    } else {
      // Register the waker to be woken when the frame changes. We will observe that it has
      // indeed changed and return Ready since we have saved the current frame at construction.
      Executor::add_waker_for_update_callback(self.state.executor.as_ptr(), ctxt.waker());
      Poll::Pending
    }
  }
}

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
    let bitmap_ptr = unsafe { self.state.graphics.newBitmap.unwrap()(width, height, bg_color.as_c_color()) };
    LCDBitmap { bitmap_ptr, state: self.state }
  }

  pub fn draw_bitmap(&self, bitmap: &LCDBitmap, x: i32, y: i32, flip: LCDBitmapFlip) {
    unsafe {
      self.state.graphics.drawBitmap.unwrap()(bitmap.bitmap_ptr, x, y, flip);
    }
  }
}
