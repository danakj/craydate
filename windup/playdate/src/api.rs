use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use playdate_sys::playdate_sys as CSystem;
use playdate_sys::PlaydateAPI as CApi;

use crate::executor::Executor;
use crate::CStr;

pub struct Api {
  pub system: System,
}
impl Api {
  pub(crate) fn new(c_api: &'static CApi, exec: *mut Executor) -> Api {
    Api {
      system: System {
        system: unsafe { &*c_api.system },
        exec,
      },
    }
  }
}

pub struct System {
  pub(crate) system: &'static CSystem,
  pub(crate) exec: *mut Executor,
}
impl System {
  /// An async function that waits for the next update step from the Playdate SDK.
  pub async fn next_update(&self) {
    self.next_update_sync().await
  }

  fn next_update_sync(&self) -> NextUpdateFuture {
    NextUpdateFuture {
      exec: self.exec,
      seen_frame: unsafe { (*self.exec).frame },
    }
  }

  pub fn log(&self, s: &CStr) {
    let exec: &mut Executor = unsafe { &mut *(self.exec as *mut Executor) };
    unsafe { exec.system.logToConsole.unwrap()(s.as_ptr()) };
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
      exec.wakers_waiting_for_update.push(ctxt.waker().clone());
      Poll::Pending
    }
  }
}
