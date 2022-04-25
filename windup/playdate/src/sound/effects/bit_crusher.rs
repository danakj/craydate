use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;

// A `BitCrusher` effect. A `BitCrusher` acts as a `SoundEffect` which can be added to a
// `SoundChannel`.
pub struct BitCrusher {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CBitCrusher>,
  amount_modulator: Option<SynthSignal>,
  undersampling_modulator: Option<SynthSignal>,
}
impl BitCrusher {
  /// Creates a new `BitCrusher` effect.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newBitCrusher.unwrap()() };
    BitCrusher {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      amount_modulator: None,
      undersampling_modulator: None,
    }
  }

  /// Sets the amount of crushing to amount.
  ///
  /// Valid values are 0 (no effect) to 1 (quantizing output to 1-bit).
  pub fn set_amount(&mut self, amount: f32) {
    unsafe { Self::fns().setAmount.unwrap()(self.cptr_mut(), amount) }
  }
  /// Sets a signal to modulate the crushing amount.
  pub fn set_amount_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| {
      // setAmountModulator() takes a mutable pointer to the modulator but there is no visible state
      // on the modulator.
      signal.as_ref().cptr() as *mut _
    });
    unsafe { Self::fns().setAmountModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.amount_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the crushing amount.
  pub fn amount_modulator(&mut self) -> Option<&SynthSignal> {
    self.amount_modulator.as_ref()
  }

  /// Sets the number of samples to repeat, quantizing the input in time.
  ///
  /// A value of 0 produces no undersampling, 1 repeats every other sample, etc.
  pub fn set_undersampling(&mut self, undersampling: f32) {
    unsafe { Self::fns().setUndersampling.unwrap()(self.cptr_mut(), undersampling) }
  }
  /// Sets a signal to modulate the undersampling amount.
  pub fn set_undersampling_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal|
      // setUndersampleModulator() takes a mutable pointer to the modulator but there is no visible
      // state on the modulator.
      signal.as_ref().cptr() as *mut _);
    unsafe { Self::fns().setUndersampleModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.undersampling_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the undersampling amount.
  pub fn undersampling_modulator(&mut self) -> Option<&SynthSignal> {
    self.undersampling_modulator.as_ref()
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut CBitCrusher {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_effect_bitcrusher {
    unsafe { &*(*CApiState::get().csound.effect).bitcrusher }
  }
}

impl Drop for BitCrusher {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeBitCrusher.unwrap()(self.cptr_mut()) }
  }
}

impl AsRef<SoundEffect> for BitCrusher {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for BitCrusher {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
