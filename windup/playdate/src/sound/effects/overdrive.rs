use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

pub struct Overdrive {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<COverdrive>,
  limit_modulator: Option<SynthSignal>,
  offset_modulator: Option<SynthSignal>,
}
impl Overdrive {
  /// Creates a new Overdrive, which acts as a SoundEffect.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newOverdrive.unwrap()() };
    Overdrive {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      limit_modulator: None,
      offset_modulator: None,
    }
  }

  /// Sets the gain of the overdrive effect.
  pub fn set_gain(&mut self, gain: f32) {
    unsafe { Self::fns().setGain.unwrap()(self.cptr(), gain) }
  }

  /// Sets the level where the amplified input clips.
  pub fn set_limit(&mut self, limit: f32) {
    unsafe { Self::fns().setLimit.unwrap()(self.cptr(), limit) }
  }
  /// Sets a signal to modulate the limit parameter.
  pub fn set_limit_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setLimitModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.limit_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the limit parameter.
  pub fn limit_modulator(&mut self) -> Option<&SynthSignal> {
    self.limit_modulator.as_ref()
  }

  /// Adds an offset to the upper and lower limits to create an asymmetric clipping.
  pub fn set_offset(&mut self, offset: f32) {
    unsafe { Self::fns().setOffset.unwrap()(self.cptr(), offset) }
  }
  /// Sets a signal to modulate the offset parameter.
  pub fn set_offset_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setOffsetModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.offset_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the offset parameter.
  pub fn offset_modulator(&mut self) -> Option<&SynthSignal> {
    self.offset_modulator.as_ref()
  }

  pub(crate) fn cptr(&self) -> *mut COverdrive {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect_overdrive {
    unsafe { &*(*CApiState::get().csound.effect).overdrive }
  }
}

impl Drop for Overdrive {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeOverdrive.unwrap()(self.cptr()) }
  }
}

impl AsRef<SoundEffect> for Overdrive {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for Overdrive {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
