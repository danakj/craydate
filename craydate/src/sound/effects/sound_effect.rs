use alloc::rc::Rc;
use alloc::rc::Weak;
use core::ptr::NonNull;

use super::super::signals::synth_signal::SynthSignal;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;

#[derive(Debug)]
enum Attachment {
  None,
  Channel(Weak<NonNull<CSoundChannel>>),
}

/// A `SoundEffect` can be attached to a `SoundChannel` to filter/mutate the sound being played on
/// it. They all include a mix modulator that allows adjusting how much to mix the `SoundEffect`
/// into the channel, between replacing the existing signal or leaving it unchanged.
///
/// There are many types which act as a `SoundEffect`. Any such type would implement
/// `AsRef<SoundEffect>` and `AsMut<SoundEffect>`. They also have `as_sound_effect()` and
/// `as_sound_effect_mut()` methods, through the `AsSoundEffect` trait, to access the `SoundEffect`
/// methods more easily.
#[derive(Debug)]
pub struct SoundEffect {
  ptr: NonNull<CSoundEffect>,
  attachment: Attachment,
  mix_modulator: Option<SynthSignal>,
}
impl SoundEffect {
  pub(crate) fn from_ptr(ptr: *mut CSoundEffect) -> Self {
    SoundEffect {
      ptr: NonNull::new(ptr).unwrap(),
      attachment: Attachment::None,
      mix_modulator: None,
    }
  }

  /// Sets the wet/dry mix for the effect.
  ///
  /// A level of 1 (full wet) replaces the input with the effect output; 0 leaves the effect out of
  /// the mix (which is useful if you’re using a delay line with taps and don’t want to hear the
  /// delay line itself).
  pub fn set_mix(&mut self, mix: f32) {
    unsafe { Self::fns().setMix.unwrap()(self.cptr_mut(), mix) }
  }

  /// Sets a signal to modulate the effect’s mix level.
  pub fn set_mix_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal|
      // setMixModulator() takes a mutable pointer to the modulator but there is no visible state on
      // the modulator.
      signal.as_ref().cptr() as *mut _);
    unsafe { Self::fns().setMixModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.mix_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the effect’s mix level.
  pub fn mix_modulator(&mut self) -> Option<&SynthSignal> {
    self.mix_modulator.as_ref()
  }

  /// Called from `SoundChannel` when the effect is attached to it, in order to update its
  /// attachment state.
  pub(crate) fn attach_to_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    match self.attachment {
      Attachment::None => {
        self.attachment = Attachment::Channel(Rc::downgrade(&channel));
        let channel_api = CApiState::get().csound.channel;
        unsafe { (*channel_api).addEffect.unwrap()(channel.as_ptr(), self.cptr_mut()) };
        Ok(())
      }
      _ => Err(Error::AlreadyAttachedError),
    }
  }

  /// Called from `SoundChannel` when the effect is detached from it, in order to update its
  /// attachment state.
  pub(crate) fn detach_from_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    match &mut self.attachment {
      Attachment::Channel(weak_ptr) if weak_ptr.ptr_eq(&Rc::downgrade(channel)) => {
        self.attachment = Attachment::None;
        let channel_api = CApiState::get().csound.channel;
        unsafe { (*channel_api).removeEffect.unwrap()(channel.as_ptr(), self.cptr_mut()) };
        Ok(())
      }
      _ => Err(Error::NotFoundError),
    }
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut CSoundEffect {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_sound_effect {
    unsafe { &*CApiState::get().csound.effect }
  }
}

impl Drop for SoundEffect {
  fn drop(&mut self) {
    match &self.attachment {
      Attachment::None => (),
      Attachment::Channel(weak_ptr) => {
        if let Some(rc_ptr) = weak_ptr.upgrade() {
          let r = self.detach_from_channel(&rc_ptr);
          assert!(r.is_ok()); // Otherwise, `self.channel` was lying.
        }
      }
    }
  }
}

pub trait AsSoundEffect: AsRef<SoundEffect> + AsMut<SoundEffect> {
  fn as_sound_effect(&self) -> &SoundEffect {
    self.as_ref()
  }
  fn as_sound_effect_mut(&mut self) -> &mut SoundEffect {
    self.as_mut()
  }
}
impl<T> AsSoundEffect for T where T: AsRef<SoundEffect> + AsMut<SoundEffect> {}
