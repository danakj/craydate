use alloc::rc::Rc;
use core::ptr::NonNull;

use super::synth_signal::{SynthSignal, SynthSignalSubclass};
use crate::capi_state::CApiState;
use crate::ctypes::*;

struct EnvelopeSubclass {
  ptr: NonNull<CSynthEnvelope>,
}
impl Drop for EnvelopeSubclass {
  fn drop(&mut self) {
    unsafe { Envelope::fns().freeEnvelope.unwrap()(self.ptr.as_ptr()) }
  }
}
impl SynthSignalSubclass for EnvelopeSubclass {}

/// An Envelope is used to modulate sounds in a `Synth`.
pub struct Envelope {
  signal: SynthSignal,
  subclass: Rc<EnvelopeSubclass>,
}
impl Envelope {
  fn from_ptr(ptr: *mut CSynthEnvelope) -> Self {
    let subclass = Rc::new(EnvelopeSubclass {
      ptr: NonNull::new(ptr).unwrap(),
    });
    let signal = SynthSignal::new(ptr as *mut CSynthSignalValue, subclass.clone());
    Envelope { signal, subclass }
  }

  /// Constructs a new `Envelope`.
  ///
  /// TODO: What are the units of `attack`? Should it be a TimeTicks?
  /// TODO: What are the units of `decay`? Should it be a TimeTicks?
  /// TODO: What are the units of `sustain`? Should it be a TimeTicks?
  /// TODO: What are the units of `release`? Should it be a TimeTicks?
  pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
    let ptr = unsafe { Self::fns().newEnvelope.unwrap()(attack, decay, sustain, release) };
    Self::from_ptr(ptr)
  }

  /// TODO: What are the units of `attack`? Should it be a TimeTicks?
  pub fn set_attack(&mut self, attack: f32) {
    unsafe { Self::fns().setAttack.unwrap()(self.cptr(), attack) }
  }
  /// TODO: What are the units of `decay`? Should it be a TimeTicks?
  pub fn set_decay(&mut self, decay: f32) {
    unsafe { Self::fns().setDecay.unwrap()(self.cptr(), decay) }
  }
  /// TODO: What are the units of `sustain`? Should it be a TimeTicks?
  pub fn set_sustain(&mut self, sustain: f32) {
    unsafe { Self::fns().setSustain.unwrap()(self.cptr(), sustain) }
  }
  /// TODO: What are the units of `release`? Should it be a TimeTicks?
  pub fn set_release(&mut self, release: f32) {
    unsafe { Self::fns().setRelease.unwrap()(self.cptr(), release) }
  }

  /// Sets whether to use legato phrasing for the envelope.
  ///
  /// If the legato flag is set, when the envelope is re-triggered before it’s released, it remains
  /// in the sustain phase instead of jumping back to the attack phase.
  pub fn set_legato(&mut self, legato: bool) {
    unsafe { Self::fns().setLegato.unwrap()(self.cptr(), legato as i32) }
  }

  /// Sets whether to start from 0 when playing a note.
  /// 
  /// If retrigger is on, the envelope always starts from 0 when a note starts playing, instead of
  /// the current value if it’s active.
  pub fn set_retrigger(&mut self, retrigger: bool) {
    unsafe { Self::fns().setRetrigger.unwrap()(self.cptr(), retrigger as i32) }
  }

  /// Return the current output value of the `Envelope`.
  pub fn get_value(&self) -> f32 {
      unsafe { Self::fns().getValue.unwrap()(self.cptr()) }
  }

  pub(crate) fn cptr(&self) -> *mut CSynthEnvelope {
    self.subclass.ptr.as_ptr() as *mut CSynthEnvelope
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_envelope {
    unsafe { &*CApiState::get().csound.envelope }
  }
}

impl AsRef<SynthSignal> for Envelope {
  fn as_ref(&self) -> &SynthSignal {
    &self.signal
  }
}
impl AsMut<SynthSignal> for Envelope {
  fn as_mut(&mut self) -> &mut SynthSignal {
    &mut self.signal
  }
}
