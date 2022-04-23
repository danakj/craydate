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
    unsafe { Self::fns().setMix.unwrap()(self.cptr(), mix) }
  }

  /// Sets a signal to modulate the effect’s mix level.
  pub fn set_mix_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setMixModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.mix_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the effect’s mix level.
  pub fn mix_modulator(&mut self) -> Option<&SynthSignal> {
    self.mix_modulator.as_ref()
  }

  pub(crate) fn attach_to_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    match self.attachment {
      Attachment::None => {
        self.attachment = Attachment::Channel(Rc::downgrade(&channel));
        let channel_api = CApiState::get().csound.channel;
        unsafe { (*channel_api).addEffect.unwrap()(channel.as_ptr(), self.cptr()) };
        Ok(())
      }
      _ => Err(Error::AlreadyAttachedError),
    }
  }

  pub(crate) fn detach_from_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    match &mut self.attachment {
      Attachment::Channel(weak_ptr) if weak_ptr.ptr_eq(&Rc::downgrade(channel)) => {
        self.attachment = Attachment::None;
        let channel_api = CApiState::get().csound.channel;
        unsafe { (*channel_api).removeEffect.unwrap()(channel.as_ptr(), self.cptr()) };
        Ok(())
      }
      _ => Err(Error::NotFoundError),
    }
  }

  pub(crate) fn cptr(&self) -> *mut CSoundEffect {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect {
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
