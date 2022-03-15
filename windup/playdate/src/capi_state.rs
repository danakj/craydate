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
}
