//! Helpers for the craydate-macro crate. Not meant to be used by human-written code.
extern crate alloc; // `alloc` is fine to use once initialize() has set up the allocator.

use alloc::boxed::Box;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::executor::Executor;
use crate::*;

#[doc(hidden)]
pub mod __private {
  use super::*;

  /// The configuration given from the `#[main]` attribute macro to the event handler.
  pub struct GameConfig {
    pub main_fn: fn(api::Api) -> Pin<Box<dyn Future<Output = !>>>,
  }

  // A placeholder to avoid exposing the type/value to craydate's dependent.
  #[repr(transparent)]
  pub struct EventHandler1(*mut CPlaydateApi);

  // A placeholder to avoid exposing the type/value to craydate's dependent.
  #[repr(transparent)]
  pub struct EventHandler2(CSystemEvent);

  // A placeholder for `u32` to avoid exposing the type/value to craydate's dependent.
  #[repr(transparent)]
  pub struct EventHandler3(u32);

  /// Called from the `#[main]` attribute macro.
  pub fn event_handler(
    eh1: EventHandler1,
    eh2: EventHandler2,
    eh3: EventHandler3,
    config: GameConfig,
  ) {
    // SAFETY: We have made a shared reference to the `CPlaydateApi`. Only refer to the object
    // through the reference hereafter. We can ensure that by never passing a pointer to the
    // `CPlaydateApi` or any pointer or reference to `CSystemApi` elsewhere.
    let api: &CPlaydateApi = unsafe { &(*eh1.0) };
    let event = eh2.0;
    let arg = eh3.0;

    match event {
      CSystemEvent::kEventInit => {
        // SAFETY: Do not allocate before the GLOBAL_ALLOCATOR is set up here, or we will crash in
        // the allocator.
        unsafe { GLOBAL_ALLOCATOR.set_system_ptr(&*api.system) };

        let capi_state = CApiState::new(api);
        // We leak this pointer so it has 'static lifetime.
        let capi_state = Box::into_raw(Box::new(capi_state));
        // The CApiState is always accessed through a shared pointer. And the CApiState is constructed
        // in initialize() and then never destroyed, so references can be 'static lifetime.
        let capi_state: &'static CApiState = unsafe { &*capi_state };
        CApiState::set_instance(capi_state);

        // We start by running the main function. This gets the future for our single execution
        // of the main function. The main function can never return (its output is `!`), so the
        // future will never be complete. We will poll() it to actually run the code in the main
        // function on the first execution of update_callback().
        Executor::set_main_future(capi_state.executor, (config.main_fn)(api::Api::new()));

        unsafe {
          capi_state.csystem.setUpdateCallback.unwrap()(
            Some(update_callback),
            core::ptr::null_mut(),
          )
        };
      }
      CSystemEvent::kEventInitLua => (),
      CSystemEvent::kEventKeyPressed => {
        CApiState::get().add_system_event(SystemEvent::SimulatorKeyPressed { keycode: arg });
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventKeyReleased => {
        CApiState::get().add_system_event(SystemEvent::SimulatorKeyReleased { keycode: arg });
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventLock => {
        CApiState::get().add_system_event(SystemEvent::WillLock);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventLowPower => {
        CApiState::get().add_system_event(SystemEvent::WillSleep);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventPause => {
        CApiState::get().add_system_event(SystemEvent::WillPause);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventResume => {
        CApiState::get().add_system_event(SystemEvent::WillResume);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventTerminate => {
        CApiState::get().add_system_event(SystemEvent::WillTerminate);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      CSystemEvent::kEventUnlock => {
        CApiState::get().add_system_event(SystemEvent::DidUnlock);
        Executor::wake_system_wakers(CApiState::get().executor);
      }
      _ => (),
    }
  }

  extern "C" fn update_callback(_: *mut c_void) -> i32 {
    // The CApiState is constructed in event_handler() and then never destroyed, so references can be
    // 'static lifetime.
    let capi = CApiState::get();

    // Drop any bitmaps from the previous frame off the ContextStack.
    capi.reset_context_stack();

    // We poll any pending futures before the frame number moves to the next frame. This allows them
    // to await the FrameWatcher and immediately be woken instead of having to skip a frame. In
    // particular this allows the main function to wait for the next frame at the top of its main loop
    // without missing the first frame.
    Executor::poll_futures(capi.executor);

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

    CApiState::get().add_system_event(SystemEvent::NextFrame {
      frame_number: capi.frame_number.get(),
      inputs: Inputs::new(
        capi.peripherals_enabled.get(),
        &capi.button_state_per_frame.get().map(|b| b.unwrap()),
      ),
    });
    Executor::wake_system_wakers(capi.executor);

    1 // Returning 0 will pause the simulator.
  }
}
