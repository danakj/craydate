//! Helpers for the playdate-macro crate. Not meant to be used by human-written code.
extern crate alloc; // `alloc` is fine to use once initialize() has set up the allocator.

pub use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use playdate_sys::playdate_sys as System;
use playdate_sys::PDSystemEvent as SystemEvent;
use playdate_sys::PlaydateAPI as Api;

use crate::*;

pub struct GameConfig {
  pub main_fn: fn(SafeApi) -> Pin<Box<dyn Future<Output = !>>>,
}

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler1(*mut Api);

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler2(SystemEvent);

// A placeholder for `u32` to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler3(u32);

struct Executor {
  system: &'static System,
  main_future: Option<Pin<Box<dyn Future<Output = !>>>>,
  poll_main: bool,
  frame: u64,
  wakers_waiting_for_update: Vec<Waker>,
}

pub fn initialize(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3, config: GameConfig) {
  let api = eh1.0;
  let event = eh2.0;
  let _arg = eh3.0;

  // SAFETY: We have made a shared reference to the System. Only refer to the object through
  // the reference hereafter. We can ensure that by never passing a pointer to the `System` or any
  // pointer or reference to the `Api` elsewhere.
  let system: &System = unsafe { &*(*api).system };

  if event == SystemEvent::kEventInit {
    // SAFETY: Do not allocate before the GLOBAL_ALLOCATOR is set up here, or we will crash
    // in the allocator.
    GLOBAL_ALLOCATOR.set_system_ptr(system);

    // We will leak this pointer so it has 'static lifetime.
    let exec_ptr = Box::into_raw(Box::new(Executor {
      system,
      main_future: None,
      poll_main: true,
      frame: 0,
      // There will only ever be a single such waker unless we introduce a spawn()
      // or similar function that has a 2nd async function running in tandem with the
      // main function (ie. when it blocks on an async thing).
      wakers_waiting_for_update: Vec::with_capacity(1),
    }));

    // We start by running the main function. This gets the future for our single execution
    // of the main function. The main function can never return (its output is `!`), so the
    // future will never be complete. We will poll() it to actually run the code in the main
    // function on the first execution of update_callback().
    let future = (config.main_fn)(SafeApi { exec: exec_ptr });
    // SAFETY: Nothing stores the executor as a reference. The main function has run and constructed
    // a future, which may have the executor pointer inside, but not a reference.
    unsafe { (*exec_ptr).main_future = Some(future) };

    unsafe { system.setUpdateCallback.unwrap()(Some(update_callback), exec_ptr as *mut c_void) };
  }
}

mod poll_main_waker {
  //! Implements a Waker that when woken will tell the executor to poll() a future again.
  //!
  //! In this case the only future tracked by the executor is the main function, unless we
  //! introduced a spawn() or similar function to run multiple async functions in tandem
  //! when each gets blocked
  use super::*;

  fn clone_fn(exec_ptr: *const ()) -> RawWaker {
    RawWaker::new(exec_ptr, &VTABLE)
  }
  fn wake_fn(exec_ptr: *const ()) {
    wake_by_ref_fn(exec_ptr);
    drop_fn(exec_ptr);
  }
  fn wake_by_ref_fn(exec_ptr: *const ()) {
    let exec: &mut Executor = unsafe { &mut *(exec_ptr as *mut Executor) };
    exec.poll_main = true;
  }
  fn drop_fn(_exec_ptr: *const ()) {}

  static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_fn, wake_fn, wake_by_ref_fn, drop_fn);

  pub(in crate::macro_helpers) fn make_waker(exec_ptr: *mut Executor) -> Waker {
    let raw_waker = RawWaker::new(exec_ptr as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
  }
}

extern "C" fn update_callback(exec_ptr: *mut c_void) -> i32 {
  let exec_ptr = exec_ptr as *mut Executor;

  {
    let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
    exec.frame += 1;
  }

  // SAFETY: Waking a waker can execute arbitrary code, so we could end up in code with access to the
  // executor, so we must drop our reference to the Executor first.
  let mut wakers = {
    let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
    core::mem::replace(&mut exec.wakers_waiting_for_update, Vec::with_capacity(1))
  };
  for w in wakers.drain(..) {
    w.wake()
  }

  let poll_main = {
    let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
    exec.poll_main
  };
  if poll_main {
    let waker = poll_main_waker::make_waker(exec_ptr);
    // SAFETY: poll() can execute arbitrary code, so we could end up in code with access to the
    // executor, so we must drop our reference to the Executor first.
    let future = {
      let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
      // This returns a reference to the inside of a Box<>, so it's not a reference into
      // the Exector, so it's okay to have that reference around while other code uses the
      // Executor.
      let option_ref = exec.main_future.as_mut();
      // Unwrap is okay because there's always a Future present, since the main function never
      // returns. The `main_future` type is only Option to split the construction of the Exector.
      option_ref.unwrap().as_mut()
    };
    // The known-to-be Pending result here is "handled" because keep the future around in the Executor in
    // order to poll it when the `waker` is woken.
    let _ = future.poll(&mut Context::from_waker(&waker));
  }

  1 // Returning 0 will pause the simulator.
}

pub struct SafeApi {
  exec: *mut Executor,
}
impl SafeApi {
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

// SAFETY: Can not hold a reference on the Executor.
unsafe fn log(exec_ptr: *mut Executor, bytes: &[u8]) {
  (*exec_ptr).system.logToConsole.unwrap()(CStr::from_bytes_with_nul(bytes).unwrap().as_ptr())
}
