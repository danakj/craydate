use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

/// The `OnePoleFilter` is a simple low/high pass filter, with a single parameter describing the
/// cutoff frequency: values above 0 (up to 1) are high-pass, values below 0 (down to -1) are
/// low-pass. A `OnePoleFilter` acts as a `SoundEffect` which can be added to a
// `SoundChannel`.
pub struct OnePoleFilter {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<COnePoleFilter>,
  parameter_modulator: Option<SynthSignal>,
}
impl OnePoleFilter {
  /// Creates a new `OnePoleFilter`.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newFilter.unwrap()() };
    OnePoleFilter {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      parameter_modulator: None,
    }
  }

  /// Sets the filter’s single parameter (cutoff frequency) to `parameter`.
  /// 
  /// Values above 0 (up to 1) are high-pass, values below 0 (down to -1) are low-pass.
  pub fn set_parameter(&mut self, parameter: f32) {
    unsafe { Self::fns().setParameter.unwrap()(self.cptr_mut(), parameter) }
  }
  /// Sets a signal to modulate the filter parameter.
  pub fn set_parameter_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| 
      // setParameterModulator() takes a mutable pointer to the modulator but there is no visible
      // state on the modulator.
      signal.as_ref().cptr() as *mut _);
    unsafe { Self::fns().setParameterModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.parameter_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the filter parameter.
  pub fn parameter_modulator(&mut self) -> Option<&SynthSignal> {
    self.parameter_modulator.as_ref()
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut COnePoleFilter {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_sound_effect_onepolefilter {
    unsafe { &*(*CApiState::get().csound.effect).onepolefilter }
  }
}

impl Drop for OnePoleFilter {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeFilter.unwrap()(self.cptr_mut()) }
  }
}

impl AsRef<SoundEffect> for OnePoleFilter {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for OnePoleFilter {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
