use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::TimeDelta;

pub struct DelayLine {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CDelayLine>,
}
impl DelayLine {
  /// Creates a new DelayLine, which acts as a SoundEffect.
  pub fn new(length: TimeDelta, stereo: bool) -> Self {
    let ptr =
      unsafe { Self::fns().newDelayLine.unwrap()(length.to_sample_frames(), stereo as i32) };
    DelayLine {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
    }
  }
  pub fn as_sound_effect(&self) -> &SoundEffect {
    &self.effect
  }
  pub fn as_sound_effect_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }

  /// Changes the length of the delay line, clearing its contents.
  pub fn set_length(&mut self, length: TimeDelta) {
    unsafe { Self::fns().setLength.unwrap()(self.cptr(), length.to_sample_frames()) }
  }
  /// Sets the feedback level of the delay line.
  pub fn set_feedback(&mut self, feedback: f32) {
    unsafe { Self::fns().setFeedback.unwrap()(self.cptr(), feedback) }
  }

  pub(crate) fn cptr(&self) -> *mut CDelayLine {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect_delayline {
    unsafe { &*(*CApiState::get().csound.effect).delayline }
  }
}

impl Drop for DelayLine {
  fn drop(&mut self) {
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeDelayLine.unwrap()(self.cptr()) }
  }
}

impl AsRef<SoundEffect> for DelayLine {
  fn as_ref(&self) -> &SoundEffect {
    &self.effect
  }
}
impl AsMut<SoundEffect> for DelayLine {
  fn as_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }
}
