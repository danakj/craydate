use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

// A two pole IIR filter, which is one of the `TwoPoleFilterType` types.
pub struct TwoPoleFilter {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CTwoPoleFilter>,
  frequency_modulator: Option<SynthSignal>,
  resonance_modulator: Option<SynthSignal>,
}
impl TwoPoleFilter {
  /// Creates a new TwoPoleFilter, which acts as a SoundEffect.
  pub fn new(filter_type: TwoPoleFilterType) -> Self {
    let ptr = unsafe { Self::fns().newFilter.unwrap()() };
    let mut f = TwoPoleFilter {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      frequency_modulator: None,
      resonance_modulator: None,
    };
    f.set_type(filter_type);
    f
  }

  /// Sets the type of the filter.
  pub fn set_type(&mut self, filter_type: TwoPoleFilterType) {
    unsafe { Self::fns().setType.unwrap()(self.cptr(), filter_type) }
  }

  /// Sets the center/corner frequency of the filter. Value is in Hz.
  pub fn set_frequency(&mut self, frequency: f32) {
    unsafe { Self::fns().setFrequency.unwrap()(self.cptr(), frequency) }
  }
  /// Sets a signal to modulate the effect’s frequency.
  ///
  /// The signal is scaled so that a value of 1.0 corresponds to half the sample rate.
  pub fn set_frequency_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setFrequencyModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.frequency_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the effect’s frequency.
  pub fn frequency_modulator(&mut self) -> Option<&SynthSignal> {
    self.frequency_modulator.as_ref()
  }

  /// Sets the filter gain.
  pub fn set_gain(&mut self, gain: f32) {
    unsafe { Self::fns().setGain.unwrap()(self.cptr(), gain) }
  }

  /// Sets the center/corner resonance of the filter. Value is in Hz.
  pub fn set_resonance(&mut self, resonance: f32) {
    unsafe { Self::fns().setResonance.unwrap()(self.cptr(), resonance) }
  }
  /// Sets a signal to modulate the effect’s filter resonance.
  pub fn set_resonance_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setResonanceModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.resonance_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the effect’s filter resonance.
  pub fn resonance_modulator(&mut self) -> Option<&SynthSignal> {
    self.resonance_modulator.as_ref()
  }

  pub(crate) fn cptr(&self) -> *mut CTwoPoleFilter {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect_twopolefilter {
    unsafe { &*(*CApiState::get().csound.effect).twopolefilter }
  }
}

impl Drop for TwoPoleFilter {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeFilter.unwrap()(self.cptr()) }
  }
}

impl AsRef<SoundEffect> for TwoPoleFilter {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for TwoPoleFilter {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
