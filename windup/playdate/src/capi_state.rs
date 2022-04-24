use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::{Cell, RefCell};
use core::ptr::NonNull;

use crate::ctypes::*;
use crate::executor::Executor;
use crate::graphics::ContextStack;
use crate::system_event::{SystemEvent, SystemEventWatcherState};

static mut GLOBAL_CAPI_STATE: Option<&'static CApiState> = None;

#[non_exhaustive]
pub(crate) struct CApiState {
  pub cdisplay: &'static CDisplayApi,
  pub csystem: &'static CSystemApi,
  pub cfile: &'static CFileApi,
  pub cgraphics: &'static CGraphicsApi,
  pub csound: &'static CSoundApi,
  pub executor: NonNull<Executor>,

  pub frame_number: Cell<u64>,
  pub peripherals_enabled: Cell<Peripherals>,
  // Tracks the button state for the current and previous frame respectively.
  pub button_state_per_frame: Cell<[Option<PDButtonsSet>; 2]>,
  pub stack: RefCell<ContextStack>,
  // Tracks how many times the stencil was set.
  pub stencil_generation: Cell<usize>,
  // Tracks how many times the font was set.
  pub font_generation: Cell<usize>,
  pub system_event_watcher_state: RefCell<Rc<SystemEventWatcherState>>,
}
impl CApiState {
  pub fn new(capi: &'static CPlaydateApi) -> CApiState {
    CApiState {
      cgraphics: unsafe { &*capi.graphics },
      csystem: unsafe { &*capi.system },
      cdisplay: unsafe { &*capi.display },
      cfile: unsafe { &*capi.file },
      csound: unsafe { &*capi.sound },
      executor: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Executor::new()))) },
      frame_number: Cell::new(0),
      peripherals_enabled: Cell::new(Peripherals::kNone),
      button_state_per_frame: Cell::new([None, None]),
      stack: RefCell::new(ContextStack::new()),
      stencil_generation: Cell::new(0),
      font_generation: Cell::new(0),
      system_event_watcher_state: RefCell::new(Rc::new(SystemEventWatcherState::new())),
    }
  }
  pub fn set_instance(capi: &'static CApiState) {
    unsafe { GLOBAL_CAPI_STATE = Some(capi) };
  }
  pub fn get() -> &'static CApiState {
    unsafe { GLOBAL_CAPI_STATE.unwrap() }
  }
  pub fn try_get() -> Option<&'static CApiState> {
    unsafe { GLOBAL_CAPI_STATE }
  }

  /// Stores the current frame's button states, and moves the previous frames' states into the next
  /// position.
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

  pub fn reset_context_stack(&self) {
    *self.stack.borrow_mut() = ContextStack::new();
  }

  pub fn add_system_event(&self, event: SystemEvent) {
    let state = self.system_event_watcher_state.borrow_mut();
    assert!(state.next_event.take().is_none());
    state.next_event.set(Some(event));
  }
}
