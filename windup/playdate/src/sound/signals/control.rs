use alloc::rc::Rc;
use core::ptr::NonNull;

use super::synth_signal::{SynthSignal, SynthSignalSubclass};
use crate::capi_state::CApiState;
use crate::ctypes::*;

struct ControlSubclass {
  ptr: NonNull<CControlSignal>,
}
impl Drop for ControlSubclass {
  fn drop(&mut self) {
    unsafe { Control::fns().freeSignal.unwrap()(self.ptr.as_ptr()) }
  }
}
impl SynthSignalSubclass for ControlSubclass {}

/// An Control is used to modulate sounds in a `Synth`.
pub struct Control {
  signal: SynthSignal,
  subclass: Rc<ControlSubclass>,
}
impl Control {
  fn from_ptr(ptr: *mut CControlSignal) -> Self {
    let subclass = Rc::new(ControlSubclass {
      ptr: NonNull::new(ptr).unwrap(),
    });
    let signal = SynthSignal::new(ptr as *mut CSynthSignalValue, subclass.clone());
    Control { signal, subclass }
  }

  /// Constructs a new control signal.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newSignal.unwrap()() };
    Self::from_ptr(ptr)
  }

  /// Clears all events from the control signal.
  pub fn clear_events(&mut self) {
    unsafe { Self::fns().clearEvents.unwrap()(self.cptr()) }
  }

  /// Adds a value to the signal’s timeline at the given step.
  ///
  /// If interpolate is true, the value is interpolated between the previous `step + value` and this
  /// one.
  pub fn add_event(&mut self, step: i32, value: f32, interpolate: bool) {
    unsafe { Self::fns().addEvent.unwrap()(self.cptr(), step, value, interpolate as i32) }
  }

  /// Removes the control event at the given step.
  pub fn remove_event(&mut self, step: i32) {
    unsafe { Self::fns().removeEvent.unwrap()(self.cptr(), step) }
  }

  /// Control signals in midi files are assigned a controller number, which describes the intent of
  /// the control. This function returns the controller number.
  /// 
  /// Returns the MIDI controller number for this ControlSignal, if it was created from a MIDI file
  /// Sequence::from_midi_file().
  pub fn midi_controller_number(&self) -> i32 {
    unsafe { Self::fns().getMIDIControllerNumber.unwrap()(self.cptr()) }
  }

  pub fn as_signal(&self) -> &SynthSignal {
    self.as_ref()
  }
  pub fn as_signal_mut(&mut self) -> &mut SynthSignal {
    self.as_mut()
  }

  fn cptr(&self) -> *mut CControlSignal {
    self.subclass.ptr.as_ptr() as *mut CControlSignal
  }
  fn fns() -> &'static playdate_sys::playdate_control_signal {
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
