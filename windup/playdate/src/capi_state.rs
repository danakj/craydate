use alloc::boxed::Box;
use core::cell::Cell;
use core::ptr::NonNull;

use crate::ctypes::*;
use crate::executor::Executor;

#[non_exhaustive]
#[derive(Debug)]
pub struct CApiState {
  pub capi: &'static CApi,
  pub cdisplay: &'static CDisplay,
  pub csystem: &'static CSystem,
  pub cgraphics: &'static CGraphics,
  pub executor: NonNull<Executor>,

  pub frame_number: Cell<u64>,
  pub peripherals_enabled: Cell<PDPeripherals>,
  // Tracks the button state for the current and previous frame respectively.
  pub button_state_per_frame: Cell<[Option<PDButtonsSet>; 2]>,
}
impl CApiState {
  pub fn new(capi: &'static CApi) -> CApiState {
    CApiState {
      cgraphics: unsafe { &*capi.graphics },
      csystem: unsafe { &*capi.system },
      cdisplay: unsafe { &*capi.display },
      capi,
      executor: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Executor::new()))) },
      frame_number: Cell::new(0),
      peripherals_enabled: Cell::new(PDPeripherals::kNone),
      button_state_per_frame: Cell::new([None, None]),
    }
  }

  /// Stores the current frame's button states, and moves the previous frames' states into the
  /// next position.
  pub fn set_current_frame_button_state(&self, buttons_set: PDButtonsSet) {
    let mut buttons = self.button_state_per_frame.take();
    // On the first frame, we push a duplicate frame.
    if let None = buttons[0] {
      buttons[0] = Some(buttons_set);
    }
    // Move the "current" slot to the "last frame" slot.
    buttons[1] = buttons[0];
    // Save the current frame.
    buttons[0] = Some(buttons_set);
    self.button_state_per_frame.set(buttons);
  }
}
