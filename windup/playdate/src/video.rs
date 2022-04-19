use core::ptr::NonNull;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::format;
use crate::null_terminated::ToNullTerminatedString;

pub struct Video {
  ptr: NonNull<CVideoPlayer>,
}
impl Video {
  /// Opens the pdv file at path and returns a new video player object for rendering its frames.
  ///
  /// If the file can not be read, the function returns an `Error::NotFoundError`.
  pub fn from_file(path: &str) -> Result<Video, Error> {
    let ptr = unsafe { Self::fns().loadVideo.unwrap()(path.to_null_terminated_utf8().as_ptr()) };
    if ptr.is_null() {
      Err(Error::NotFoundError)
    } else {
      Ok(Video {
        ptr: NonNull::new(ptr).unwrap(),
      })
    }
  }

  /// Sets the rendering destination for the video player to the screen.
  pub fn set_context_is_screen(&mut self) {
    unsafe { Self::fns().useScreenContext.unwrap()(self.cptr()) }
  }

  // TODO: setContext to bitmap.
  // TODO: getContext

  /// Renders frame number `n` into the current context.
  pub fn render_frame(&self, n: i32) -> Result<(), Error> {
    let r = unsafe { Self::fns().renderFrame.unwrap()(self.cptr(), n) };
    if r != 0 {
      Ok(())
    } else {
      let msg = unsafe {
        crate::null_terminated::parse_null_terminated_utf8(Self::fns().getError.unwrap()(
          self.cptr(),
        ))
      };
      match msg {
        Ok(err) => Err(format!("render_frame: {}", err).into()),
        Err(err) => Err(format!("render_frame: unknown error ({})", err).into()),
      }
    }
  }

  fn info(&self) -> (i32, i32, f32, i32, i32) {
    let mut width = 0;
    let mut height = 0;
    let mut frame_rate = 0.0;
    let mut frame_count = 0;
    let mut current_frame = 0;
    unsafe { Self::fns().getInfo.unwrap()(self.cptr(), &mut width, &mut height, &mut frame_rate, &mut frame_count, &mut current_frame) };
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

  pub(crate) fn cptr(&self) -> *mut CVideoPlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_video {
    unsafe { &*CApiState::get().cgraphics.video }
  }
}

impl Drop for Video {
  fn drop(&mut self) {
    unsafe { Self::fns().freePlayer.unwrap()(self.cptr()) }
  }
}
