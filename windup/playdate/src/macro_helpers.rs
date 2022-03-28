//! Helpers for the playdate-macro crate. Not meant to be used by human-written code.
extern crate alloc; // `alloc` is fine to use once initialize() has set up the allocator.

pub use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::executor::Executor;
use crate::*;

pub struct GameConfig {
  pub main_fn: fn(api::Api) -> Pin<Box<dyn Future<Output = !>>>,
}

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler1(*mut CPlaydateApi);

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

  // SAFETY: We have made a shared reference to the `CPlaydateApi`. Only refer to the object through
  // the reference hereafter. We can ensure that by never passing a pointer to the `CPlaydateApi`
  // or any pointer or reference to `CSystemApi` elsewhere.
  let api: &CPlaydateApi = unsafe { &(*api_ptr) };
  let system: &CSystemApi = unsafe { &(*api.system) };

  if event == CSystemEvent::kEventInit {
    // SAFETY: Do not allocate before the GLOBAL_ALLOCATOR is set up here, or we will crash
    // in the allocator.
    GLOBAL_ALLOCATOR.set_system_ptr(system);
    crate::debug::initialize(system);

    // We leak this pointer so it has 'static lifetime.
    let capi = Box::into_raw(Box::new(CApiState::new(api)));
    // The CApiState is always accessed through a shared pointer. And the CApiState is constructed
    // in initialize() and then never destroyed, so references can be 'static lifetime.
    let capi: &'static CApiState = unsafe { &*capi };

    // We start by running the main function. This gets the future for our single execution
    // of the main function. The main function can never return (its output is `!`), so the
    // future will never be complete. We will poll() it to actually run the code in the main
    // function on the first execution of update_callback().

    // TODO: should exec_ptr be constructed internally here?
    let api = api::Api::new(capi);

    Executor::set_main_future(capi.executor.as_ptr(), (config.main_fn)(api));

    unsafe {
      system.setUpdateCallback.unwrap()(
        Some(update_callback),
        capi as *const CApiState as *mut c_void,
      )
    };
  }
}

extern "C" fn update_callback(capi_ptr: *mut c_void) -> i32 {
  // The CApiState is constructed in initialize() and then never destroyed, so references can be
  // 'static lifetime.
  let capi: &'static CApiState = unsafe { &*(capi_ptr as *const CApiState) };
  let exec_ptr = capi.executor.as_ptr();

  // Drop any bitmaps from the previous frame off the ContextStack.
  capi.reset_context_stack();

  // We poll any pending futures before the frame number moves to the next frame. This allows them
  // to await the FrameWatcher and immediately be woken instead of having to skip a frame. In
  // particular this allows the main function to wait for the next frame at the top of its main loop
  // without missing the first frame.
  Executor::poll_futures(exec_ptr);

  capi.frame_number.set(capi.frame_number.get() + 1);

  // Capture input state which will be returned from any futures waiting for the update_callback().
  // So this must happen before we wake those futures.

  let buttons_set = unsafe {
    let mut set = PDButtonsSet {
      current: CButtons(0),
      pushed: CButtons(0),
      released: CButtons(0),
    };
    capi.csystem.getButtonState.unwrap()(&mut set.current, &mut set.pushed, &mut set.released);
    set
  };
  capi.set_current_frame_button_state(buttons_set);

  let exec: &mut Executor = unsafe { &mut *(exec_ptr) };
  let mut wakers = core::mem::replace(&mut exec.wakers_for_update_callback, Vec::with_capacity(1));
  drop(exec);

  for w in wakers.drain(..) {
    // SAFETY: Waking a waker can execute arbitrary code, so we could end up in code with access to the
    // executor, so we have dropped our reference to the Executor first.
    w.wake()
  }

  1 // Returning 0 will pause the simulator.
}
