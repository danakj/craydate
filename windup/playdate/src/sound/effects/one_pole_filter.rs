use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

pub struct OnePoleFilter {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<COnePoleFilter>,
  parameter_modulator: Option<SynthSignal>,
}
impl OnePoleFilter {
  /// Creates a new OnePoleFilter, which acts as a SoundEffect.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newFilter.unwrap()() };
    OnePoleFilter {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      parameter_modulator: None,
    }
  }
  pub fn as_sound_effect(&self) -> &SoundEffect {
    &self.effect
  }
  pub fn as_sound_effect_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }

  /// Sets the filter’s single parameter (cutoff frequency) to `parameter`.
  /// 
  /// Values above 0 (up to 1) are high-pass, values below 0 (down to -1) are low-pass.
  pub fn set_parameter(&mut self, parameter: f32) {
    unsafe { Self::fns().setParameter.unwrap()(self.cptr(), parameter) }
  }
  /// Sets a signal to modulate the filter parameter.
  pub fn set_parameter_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setParameterModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.parameter_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the filter parameter.
  pub fn parameter_modulator(&mut self) -> Option<&SynthSignal> {
    self.parameter_modulator.as_ref()
  }

  pub(crate) fn cptr(&self) -> *mut COnePoleFilter {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect_onepolefilter {
    unsafe { &*(*CApiState::get().csound.effect).onepolefilter }
  }
}

impl Drop for OnePoleFilter {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeFilter.unwrap()(self.cptr()) }
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
