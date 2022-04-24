use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

// A ring modulator effect. A `RingModulator` acts as a `SoundEffect` which can be added to a
// `SoundChannel`.
pub struct RingModulator {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CRingModulator>,
  frequency_modulator: Option<SynthSignal>,
}
impl RingModulator {
  /// Creates a new `RingModulator`.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newRingmod.unwrap()() };
    RingModulator {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      frequency_modulator: None,
    }
  }

  /// Sets the frequency of the modulation signal.
  pub fn set_frequency(&mut self, frequency: f32) {
    unsafe { Self::fns().setFrequency.unwrap()(self.cptr(), frequency) }
  }
  /// Sets a signal to modulate the frequency of the ring modulator.
  pub fn set_frequency_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setFrequencyModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.frequency_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the frequency of the ring modulator.
  pub fn frequency_modulator(&mut self) -> Option<&SynthSignal> {
    self.frequency_modulator.as_ref()
  }

  pub(crate) fn cptr(&self) -> *mut CRingModulator {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_effect_ringmodulator {
    unsafe { &*(*CApiState::get().csound.effect).ringmodulator }
  }
}

impl Drop for RingModulator {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeRingmod.unwrap()(self.cptr()) }
  }
}

impl AsRef<SoundEffect> for RingModulator {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for RingModulator {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
