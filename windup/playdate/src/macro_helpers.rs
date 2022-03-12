//! Helpers for the playdate-macro crate. Not meant to be used by human-written code.
extern crate alloc; // `alloc` is fine to use once initialize() has set up the allocator.

pub use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;

use playdate_sys::playdate_sys as CSystem;
use playdate_sys::PDSystemEvent as CSystemEvent;
use playdate_sys::PlaydateAPI as CApi;

use crate::executor::Executor;
use crate::*;

pub struct GameConfig {
  pub main_fn: fn(api::Api) -> Pin<Box<dyn Future<Output = !>>>,
}

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler1(*mut CApi);

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler2(CSystemEvent);

// A placeholder for `u32` to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler3(u32);

pub fn initialize(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3, config: GameConfig) {
  let api_ptr = eh1.0;
  let event = eh2.0;
  let _arg = eh3.0;

  // SAFETY: We have made a shared reference to the `CApi`. Only refer to the object through
  // the reference hereafter. We can ensure that by never passing a pointer to the `CApi`
  // or any pointer or reference to `CSystem` elsewhere.
  let api: &CApi = unsafe { &(*api_ptr) };
  let system: &CSystem = unsafe { &(*api.system) };

  if event == CSystemEvent::kEventInit {
    // SAFETY: Do not allocate before the GLOBAL_ALLOCATOR is set up here, or we will crash
    // in the allocator.
    GLOBAL_ALLOCATOR.set_system_ptr(system);

    // We will leak this pointer so it has 'static lifetime.
    let exec_ptr = Box::into_raw(Box::new(Executor::new(system)));

    // We start by running the main function. This gets the future for our single execution
    // of the main function. The main function can never return (its output is `!`), so the
    // future will never be complete. We will poll() it to actually run the code in the main
    // function on the first execution of update_callback().

    // TODO: should exec_ptr be constructed internally here?
    let api = api::Api::new(api, exec_ptr);

    Executor::set_main_future(exec_ptr, (config.main_fn)(api));

    unsafe { system.setUpdateCallback.unwrap()(Some(update_callback), exec_ptr as *mut c_void) };
  }
}

extern "C" fn update_callback(exec_ptr: *mut c_void) -> i32 {
  let exec_ptr = exec_ptr as *mut Executor;

  unsafe { (*exec_ptr).frame += 1 };

  let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
  let mut wakers = core::mem::replace(&mut exec.wakers_waiting_for_update, Vec::with_capacity(1));
  drop(exec);

  for w in wakers.drain(..) {
    // SAFETY: Waking a waker can execute arbitrary code, so we could end up in code with access to the
    // executor, so we have dropped our reference to the Executor first.
    w.wake()
  }

  // This happens _after_ waking wakers looking for the update_callback(), because otherwise they would
  // immediately hear that the next update happened, but they wouldn't actually be able to observe it. The
  // other option would be to poll pending futures before updating the frame number, as if they ran just
  // before the current frame.
  Executor::poll_futures(exec_ptr);

  1 // Returning 0 will pause the simulator.
}
