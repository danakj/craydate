use alloc::format;
use core::cell::Cell;
use core::ptr::NonNull;

use super::bitmap::BitmapRef;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Context {
  None,
  Screen,
  Bitmap(NonNull<CBitmap>),
}

/// A Video file that can be rendered into the display or a `Bitmap`.
pub struct Video {
  ptr: NonNull<CVideoPlayer>,
  context: Cell<Context>,
}
impl Video {
  /// Opens the `.pdv` file at path and returns a new video player object for rendering its frames.
  ///
  /// If the file can not be read, the function returns an `Error::NotFoundError`.
  pub fn from_file(path: &str) -> Result<Video, Error> {
    let ptr = unsafe { Self::fns().loadVideo.unwrap()(path.to_null_terminated_utf8().as_ptr()) };
    if ptr.is_null() {
      Err(Error::NotFoundError)
    } else {
      Ok(Video {
        context: Cell::new(Context::None),
        ptr: NonNull::new(ptr).unwrap(),
      })
    }
  }

  /// Returns an error with human-readable text describing the most recent Video error.
  fn get_render_error(&self, fn_name: &str) -> Error {
    let msg = unsafe {
      // getError() takes a mutable pointer but does not write to the video object.
      crate::null_terminated::parse_null_terminated_utf8(Self::fns().getError.unwrap()(
        self.cptr() as *mut _
      ))
    };
    match msg {
      Ok(err) => format!("{}: {}", fn_name, err).into(),
      Err(err) => format!(
        "{}: unable to parse UTF-8 error string from Playdate. {}",
        fn_name, err
      )
      .into(),
    }
  }

  /// Renders frame number `n` into the screen.
  pub fn render_frame_to_screen(&self, n: i32) -> Result<(), Error> {
    if self.context.get() != Context::Screen {
      // useScreenContext() writes to the video object, to change its context, but we don't expose
      // that change in the API, since we don't call getContext(). So we can treat this as interior
      // mutability.
      unsafe { Self::fns().useScreenContext.unwrap()(self.cptr() as *mut _) }
      self.context.set(Context::Screen);
    }

    // renderFrame() reads from the video but takes a mutable pointer.
    if unsafe { Self::fns().renderFrame.unwrap()(self.cptr() as *mut _, n) } == 0 {
      return Err(self.get_render_error("render_frame_to_screen"));
    }

    return Ok(());
  }

  /// Renders frame number `n` into the `bitmap`.
  pub fn render_frame_to_bitmap(&self, n: i32, bitmap: &mut BitmapRef) -> Result<(), Error> {
    if self.context.get() != Context::Bitmap(NonNull::new(bitmap.cptr_mut()).unwrap()) {
      // setContext() writes to the video object, to change its context, but we don't expose that
      // change in the API, since we don't call getContext(). So we can treat this as interior
      // mutability.
      if unsafe { Self::fns().setContext.unwrap()(self.cptr() as *mut _, bitmap.cptr_mut()) } == 0 {
        return Err(self.get_render_error("render_frame_to_bitmap"));
      }
    }

    // renderFrame() reads from the video but takes a mutable pointer.
    if unsafe { Self::fns().renderFrame.unwrap()(self.cptr() as *mut _, n) } == 0 {
      return Err(self.get_render_error("render_frame_to_bitmap"));
    }

    return Ok(());
  }

  fn info(&self) -> (i32, i32, f32, i32, i32) {
    let mut width = 0;
    let mut height = 0;
    let mut frame_rate = 0.0;
    let mut frame_count = 0;
    let mut current_frame = 0;
    unsafe {
      // getInfo() reads from the video but takes a mutable pointer.
      Self::fns().getInfo.unwrap()(
        self.cptr() as *mut _,
        &mut width,
        &mut height,
        &mut frame_rate,
        &mut frame_count,
        &mut current_frame,
      )
    };
    (width, height, frame_rate, frame_count, current_frame)
  }

  /// The width that the video renders.
  pub fn out_width(&self) -> i32 {
    let (width, _, _, _, _) = self.info();
    width
  }
  /// The height that the video renders.
  pub fn out_height(&self) -> i32 {
    let (_, height, _, _, _) = self.info();
    height
  }
  /// The frame rate that the video renders.
  pub fn frame_rate(&self) -> f32 {
    let (_, _, frame_rate, _, _) = self.info();
    frame_rate
  }
  /// The number of frames in the video.
  pub fn frame_count(&self) -> i32 {
    let (_, _, _, frame_count, _) = self.info();
    frame_count
  }
  /// The current frame of the video.
  pub fn current_frame(&self) -> i32 {
    let (_, _, _, _, current_frame) = self.info();
    current_frame
  }

  pub(crate) fn cptr(&self) -> *const CVideoPlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CVideoPlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_video {
    unsafe { &*CApiState::get().cgraphics.video }
  }
}

impl Drop for Video {
  fn drop(&mut self) {
    unsafe { Self::fns().freePlayer.unwrap()(self.cptr_mut()) }
  }
}
