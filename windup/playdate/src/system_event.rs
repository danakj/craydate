use alloc::rc::Rc;
use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::capi_state::CApiState;
use crate::executor::Executor;
use crate::inputs::Inputs;

/// Playdate system events.
#[derive(Debug)]
pub enum SystemEvent {
  /// Event when the next frame should be prepared for display. Handle this event by running the
  /// game's update and draw routines.
  NextFrame {
    /// The current frame number, which is monotonically increasing.
    frame_number: u64,
    /// All input events since the last frame, along with current input states.
    inputs: Inputs,
  },
  /// Event when the player chooses to exit the game via the System Menu or Menu button.
  WillTerminate,
  /// Event before the device goes to low-power sleep mode because of a low battery.
  WillSleep,
  /// Event before the system pauses the game.
  ///
  /// In the current version of Playdate OS, this only happens when the device’s Menu button is
  /// pushed. Handling this event allows your game to take special action when it is paused, e.g.,
  /// updating the menu image.
  WillPause,
  /// Event before the system resumes the game.
  WillResume,
  /// Event if your game is running on the Playdate when the device is locked.
  ///
  /// Implementing this function allows your game to take special action when the Playdate is
  /// locked, e.g., saving state.
  WillLock,
  // Event if your game is running on the Playdate when the device is unlocked.
  DidUnlock,
  /// Event when a key is pressed in the simulator. Does not occur on device.
  SimulatorKeyPressed {
    /// The pressed keycode.
    keycode: u32,
  },
  /// Event when a key is released in the simulator. Does not occur on device.
  SimulatorKeyReleased {
    /// The released keycode.
    keycode: u32,
  },
  /// A system callback is active, and the game can execute their registered closure for it by
  /// running their `Callbacks` object(s).
  Callback,
}

/// An object shared between the global `CApiState` and any `SystemEventWatcher` objects, where new
/// system events are placed in order for the `Future` returned from a `SystemEventWatcher` to find
/// them.
pub(crate) struct SystemEventWatcherState {
  /// The system event which will be returned from `SystemEventWatcher::next()`. It's normally
  /// `None` until `next()` is called and yields to the Playdate system which will generate the next
  /// event.
  pub next_event: Cell<Option<SystemEvent>>,
}
impl SystemEventWatcherState {
  pub(crate) fn new() -> Self {
    SystemEventWatcherState {
      next_event: Cell::new(None),
    }
  }
}

/// An object used to watch for the next system event. Call `next()` to get the next event when it
/// is ready.
pub struct SystemEventWatcher {
  pub(crate) state: Rc<SystemEventWatcherState>,
}
impl SystemEventWatcher {
  pub(crate) fn new() -> Self {
    let state = CApiState::get().system_event_watcher_state.borrow().clone();
    SystemEventWatcher { state }
  }

  /// Runs until the next frame from the Playdate device, then returns the frame number.
  ///
  /// This function returns after the Playdate device calls the "update callback" to signify that
  /// the game should perform updates for the next frame to be displayed.
  pub async fn next(&self) -> SystemEvent {
    self.next_impl().await
  }
  fn next_impl(&self) -> SystemEventFuture {
    SystemEventFuture { watcher: self }
  }
}

/// A future for which poll() waits for the next system event, then returns Complete.
struct SystemEventFuture<'a> {
  watcher: &'a SystemEventWatcher,
}

impl Future for SystemEventFuture<'_> {
  type Output = SystemEvent;

  fn poll(self: Pin<&mut Self>, ctxt: &mut Context<'_>) -> Poll<Self::Output> {
    match self.watcher.state.next_event.take() {
      Some(event) => Poll::Ready(event),
      None => {
        // Register the waker to be woken when an event occurs. We were polled and nothing had
        // happened yet.
        Executor::add_waker_for_system_event(CApiState::get().executor.as_ptr(), ctxt.waker());
        Poll::Pending
      }
    }
  }
}
