use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::ctypes::*;
use crate::CStr;
use crate::capi_state::CApiState;
use crate::executor::Executor;

pub struct Api {
  pub system: System,
  pub graphics: Graphics,
}
impl Api {
  pub(crate) fn new(state: &'static CApiState) -> Api {
    Api {
      system: System {
        state,
      },
      graphics: Graphics {
        state,
      },
    }
  }
}

pub struct System {
  pub(crate) state: &'static CApiState,
}
impl System {
  /// An async function that waits for the next update step from the Playdate SDK.
  pub async fn next_update(&self) {
    self.next_update_sync().await
  }

  fn next_update_sync(&self) -> NextUpdateFuture {
    NextUpdateFuture {
      exec: self.state.executor.as_ptr(),
      seen_frame: unsafe { self.state.executor.as_ref().frame },
    }
  }

  pub fn log(&self, s: &CStr) {
    unsafe { self.state.system.logToConsole.unwrap()(s.as_ptr()) };
  }
}

/// A future for which poll() waits for the next update, then returns Complete.
struct NextUpdateFuture {
  exec: *mut Executor,
  seen_frame: u64,
}

impl Future for NextUpdateFuture {
  type Output = ();

  fn poll(self: Pin<&mut Self>, ctxt: &mut Context<'_>) -> Poll<()> {
    let frame = {
      let exec: &mut Executor = unsafe { &mut *(self.exec as *mut Executor) };
      exec.frame
    };

    if frame > self.seen_frame {
      Poll::Ready(())
    } else {
      // Register the waker to be woken when the frame changes. We will observe that it has
      // indeed changed and return Ready since we have saved the current frame at construction.
      let exec: &mut Executor = unsafe { &mut *(self.exec as *mut Executor) };
      exec.wakers_for_update_callback.push(ctxt.waker().clone());
      Poll::Pending
    }
  }
}

pub struct Graphics {
  pub(crate) state: &'static CApiState,
}
impl Graphics {
  pub fn clear(&self, color: LCDSolidColor) {
    unsafe { self.state.graphics.clear.unwrap()(color.0 as usize) };
  }
}
