use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::ffi::c_void;
use core::ptr::NonNull;

use super::synth_signal::{SynthSignal, SynthSignalSubclass};
use crate::capi_state::CApiState;
use crate::ctypes::*;

struct LfoFunctionData {
  f: Box<dyn FnMut() -> f32>,
}

struct LfoSubclass {
  ptr: NonNull<CSynthLfo>,
  function_data: RefCell<Option<LfoFunctionData>>,
}
impl Drop for LfoSubclass {
  fn drop(&mut self) {
    unsafe { Lfo::fns().freeLFO.unwrap()(self.ptr.as_ptr()) }
  }
}
impl SynthSignalSubclass for LfoSubclass {}

pub enum LfoFixedFunction {
  Square,
  Triangle,
  Sine,
  SampleAndHold,
  SawtoothUp,
  SawtoothDown,
}
impl LfoFixedFunction {
  fn to_c(self) -> CSynthLfoType {
    match self {
      Self::Square => CSynthLfoType::kLFOTypeSquare,
      Self::Triangle => CSynthLfoType::kLFOTypeTriangle,
      Self::Sine => CSynthLfoType::kLFOTypeSine,
      Self::SampleAndHold => CSynthLfoType::kLFOTypeSampleAndHold,
      Self::SawtoothUp => CSynthLfoType::kLFOTypeSawtoothUp,
      Self::SawtoothDown => CSynthLfoType::kLFOTypeSawtoothDown,
    }
  }
}

/// An Lfo is used to modulate sounds in a `Synth` with a function.
pub struct Lfo {
  signal: SynthSignal,
  subclass: Rc<LfoSubclass>,
}
impl Lfo {
  fn from_ptr(ptr: *mut CSynthLfo) -> Self {
    let subclass = Rc::new(LfoSubclass {
      ptr: NonNull::new(ptr).unwrap(),
      function_data: RefCell::new(None),
    });
    let signal = SynthSignal {
      ptr: NonNull::new(ptr as *mut CSynthSignalValue).unwrap(),
      _subclass: subclass.clone(),
    };
    Lfo { signal, subclass }
  }

  /// Constructs a new LFO with the given shape. See `set_fixed_function()`.
  pub fn new_fixed_function(
    lfo_type: LfoFixedFunction,
    rate: f32,
    phase: f32,
    center: f32,
    depth: f32,
  ) -> Self {
    let ptr = unsafe { Self::fns().newLFO.unwrap()(CSynthLfoType::kLFOTypeSine) };
    let mut lfo = Self::from_ptr(ptr);
    lfo.set_fixed_function(lfo_type, rate, phase, center, depth);
    lfo
  }
  /// Constructs a new LFO with an arpeggio. See `set_arpeggiation()`.
  pub fn new_arpeggiation(steps: &[f32]) -> Self {
    let ptr = unsafe { Self::fns().newLFO.unwrap()(CSynthLfoType::kLFOTypeArpeggiator) };
    let mut lfo = Self::from_ptr(ptr);
    lfo.set_arpeggiation(steps);
    lfo
  }
  /// Constructs a new LFO with a custom function. See `set_user_function()`.
  pub fn new_user_function(
    &mut self,
    interpolate: bool,
    f: impl FnMut() -> f32 + Send + 'static,
  ) -> Self {
    let ptr = unsafe { Self::fns().newLFO.unwrap()(CSynthLfoType::kLFOTypeFunction) };
    let mut lfo = Self::from_ptr(ptr);
    lfo.set_user_function(interpolate, f);
    lfo
  }

  /// Sets the LFO to the given fixed function shape.
  ///
  /// The `rate` is in cycles per second.
  pub fn set_fixed_function(
    &mut self,
    lfo_type: LfoFixedFunction,
    rate: f32,
    phase: f32,
    center: f32,
    depth: f32,
  ) {
    unsafe { Lfo::fns().setType.unwrap()(self.cptr(), lfo_type.to_c()) };
    unsafe { Lfo::fns().setRate.unwrap()(self.cptr(), rate) };
    unsafe { Lfo::fns().setPhase.unwrap()(self.cptr(), phase) };
    unsafe { Lfo::fns().setCenter.unwrap()(self.cptr(), center) };
    unsafe { Lfo::fns().setDepth.unwrap()(self.cptr(), depth) };
  }

  /// Sets the LFO type to arpeggio, where the given values are in half-steps from the center note.
  ///
  /// For example, the sequence (0, 4, 7, 12) plays the notes of a major chord.
  pub fn set_arpeggiation(&mut self, steps: &[f32]) {
    unsafe { Lfo::fns().setType.unwrap()(self.cptr(), CSynthLfoType::kLFOTypeArpeggiator) };
    unsafe {
      Lfo::fns().setArpeggiation.unwrap()(
        self.cptr(),
        steps.len() as i32,
        steps.as_ptr() as *mut f32,
      )
    };
  }

  /// Provides a custom function for LFO values.
  ///
  /// TODO: What is `interpolate`?
  /// TODO: Does `f` need access to the `SynthLfo`?
  pub fn set_user_function(&mut self, interpolate: bool, f: impl FnMut() -> f32 + Send + 'static) {
    unsafe { Lfo::fns().setType.unwrap()(self.cptr(), CSynthLfoType::kLFOTypeFunction) };
    unsafe extern "C" fn c_func(_clfo: *mut CSynthLfo, data: *mut c_void) -> f32 {
      let data = data as *mut LfoFunctionData;
      ((*data).f)()
    }
    // We store the LfoFunctionData inside the LfoSubclass, which will live as long as the Lfo is
    // running, even after the Lfo is dropped. Then we grab a pointer to it there, after it was
    // moved into place on the heap.
    *self.subclass.function_data.borrow_mut() = Some(LfoFunctionData { f: Box::new(f) });
    let data_ptr = unsafe {
      self.subclass.function_data.borrow_mut().as_mut().unwrap_unchecked() as *mut LfoFunctionData
    };
    unsafe {
      Lfo::fns().setFunction.unwrap()(
        self.cptr(),
        Some(c_func),
        data_ptr as *mut c_void,
        interpolate as i32,
      )
    }
  }

  // TODO: setGlobal in a future update.

  /// Return the current output value of the LFO.
  pub fn get_value(&self) -> f32 {
    unsafe { Self::fns().getValue.unwrap()(self.cptr()) }
  }

  pub fn as_signal(&self) -> &SynthSignal {
    self.as_ref()
  }
  pub fn as_signal_mut(&mut self) -> &mut SynthSignal {
    self.as_mut()
  }

  fn cptr(&self) -> *mut CSynthLfo {
    self.subclass.ptr.as_ptr() as *mut CSynthLfo
  }
  fn fns() -> &'static playdate_sys::playdate_sound_lfo {
    unsafe { &*CApiState::get().csound.lfo }
  }
}

impl AsRef<SynthSignal> for Lfo {
  fn as_ref(&self) -> &SynthSignal {
    &self.signal
  }
}
impl AsMut<SynthSignal> for Lfo {
  fn as_mut(&mut self) -> &mut SynthSignal {
    &mut self.signal
  }
}
