use alloc::rc::Rc;
use core::ptr::NonNull;

use super::synth_signal::{SynthSignal, SynthSignalSubclass};
use crate::capi_state::CApiState;
use crate::ctypes::*;

/// A borrow of a `Control`.
pub struct ControlRef {
  ptr: NonNull<CControlSignal>,
}
impl ControlRef {
  pub(crate) fn from_ptr(ptr: NonNull<CControlSignal>) -> Self {
    ControlRef { ptr }
  }

  /// Clears all events from the control signal.
  pub fn clear_events(&mut self) {
    unsafe { Control::fns().clearEvents.unwrap()(self.cptr_mut()) }
  }

  /// Adds a value to the signal’s timeline at the given step.
  ///
  /// If interpolate is true, the value is interpolated between the previous `step + value` and this
  /// one.
  pub fn add_event(&mut self, step: i32, value: f32, interpolate: bool) {
    unsafe { Control::fns().addEvent.unwrap()(self.cptr_mut(), step, value, interpolate as i32) }
  }

  /// Removes the control event at the given step.
  pub fn remove_event(&mut self, step: i32) {
    unsafe { Control::fns().removeEvent.unwrap()(self.cptr_mut(), step) }
  }

  /// Control signals in midi files are assigned a controller number, which describes the intent of
  /// the control. This function returns the controller number.
  ///
  /// Returns the MIDI controller number for this ControlSignal, if it was created from a MIDI file
  /// via `Sequence::from_midi_file()`.
  pub fn midi_controller_number(&self) -> i32 {
    // getMIDIControllerNumber() takes a mutable pointer but it doesn't change any visible state.
    unsafe { Control::fns().getMIDIControllerNumber.unwrap()(self.cptr() as *mut _) }
  }

  pub(crate) fn cptr(&self) -> *const CControlSignal {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CControlSignal {
    self.ptr.as_ptr()
  }
}

/// Holds (refcounted) ownership of the C Api object inside the SynthSignal.
struct ControlSubclass {
  ptr: NonNull<CControlSignal>,
}
impl Drop for ControlSubclass {
  fn drop(&mut self) {
    unsafe { Control::fns().freeSignal.unwrap()(self.ptr.as_ptr()) }
  }
}
impl SynthSignalSubclass for ControlSubclass {}

/// A `Control` signal object is used for automating effect parameters, channel pan and level, etc.
pub struct Control {
  cref: ControlRef,
  signal: SynthSignal,
  _subclass: Rc<ControlSubclass>,
}
impl Control {
  fn from_ptr(ptr: *mut CControlSignal) -> Self {
    let subclass = Rc::new(ControlSubclass {
      ptr: NonNull::new(ptr).unwrap(),
    });
    let signal = SynthSignal::new(ptr as *mut CSynthSignalValue, subclass.clone());
    Control {
      cref: ControlRef::from_ptr(NonNull::new(ptr).unwrap()),
      signal,
      _subclass: subclass,
    }
  }

  /// Constructs a new control signal.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newSignal.unwrap()() };
    Self::from_ptr(ptr)
  }

  pub(crate) fn fns() -> &'static craydate_sys::playdate_control_signal {
    unsafe { &*CApiState::get().csound.controlsignal }
  }
}

impl AsRef<SynthSignal> for Control {
  fn as_ref(&self) -> &SynthSignal {
    &self.signal
  }
}
impl AsMut<SynthSignal> for Control {
  fn as_mut(&mut self) -> &mut SynthSignal {
    &mut self.signal
  }
}

impl core::ops::Deref for Control {
  type Target = ControlRef;

  fn deref(&self) -> &Self::Target {
    &self.cref
  }
}
impl core::ops::DerefMut for Control {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.cref
  }
}
impl AsRef<ControlRef> for Control {
  fn as_ref(&self) -> &ControlRef {
    self
  }
}
impl AsMut<ControlRef> for Control {
  fn as_mut(&mut self) -> &mut ControlRef {
    self
  }
}
impl core::borrow::Borrow<ControlRef> for Control {
  fn borrow(&self) -> &ControlRef {
    self
  }
}
impl core::borrow::BorrowMut<ControlRef> for Control {
  fn borrow_mut(&mut self) -> &mut ControlRef {
    self
  }
}
