use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::capi_state::CApiState;
use crate::executor::Executor;
use crate::graphics::Graphics;
use crate::time::TimeTicks;

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
    FrameWatcher { state: self.state }
  }

  /// Prints a string to the Playdate console, as well as to stdout.
  pub fn log<S: AsRef<str>>(&self, s: S) {
    crate::debug::log(s)
  }

  /// Prints an error string in red to the Playdate console, and pauses Playdate. Also prints the
  /// string to stdout.
  pub fn error<S: AsRef<str>>(&self, s: S) {
    crate::debug::error(s);
  }

  /// Returns the current time in milliseconds.
  pub fn get_current_time(&self) -> TimeTicks {
    TimeTicks::from(unsafe { self.state.system.getCurrentTimeMilliseconds.unwrap()() })
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
