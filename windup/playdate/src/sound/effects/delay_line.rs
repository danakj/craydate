use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::sources::delay_line_tap::DelayLineTap;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::TimeDelta;

#[derive(Debug)]
pub struct DelayLine {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CDelayLine>,
  taps: Vec<DelayLineTap>,
}
impl DelayLine {
  /// Creates a new DelayLine, which acts as a SoundEffect.
  pub fn new(length: TimeDelta, stereo: bool) -> Self {
    let ptr =
      unsafe { Self::fns().newDelayLine.unwrap()(length.to_sample_frames(), stereo as i32) };
    DelayLine {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      taps: Vec::new(),
    }
  }
  // TODO: Make this an AsSoundEffect trait like for SoundChannel.
  pub fn as_sound_effect(&self) -> &SoundEffect {
    &self.effect
  }
  pub fn as_sound_effect_mut(&mut self) -> &mut SoundEffect {
    &mut self.effect
  }

  /// Adds a new tap on the DelayLine, at the given time position within the `DelayLine`.
  ///
  /// `delay` must be less than or equal to the length of the `DelayLine`.
  ///
  /// # Return
  /// If successful, returns a borrow on the new `DelayLineTap`. If the tap was not able to be added,
  /// then `None` is returned.
  pub fn add_tap(&mut self, delay: TimeDelta) -> Option<&mut DelayLineTap> {
    let tap = DelayLineTap::new(self, delay)?;
    self.taps.push(tap);
    Some(unsafe { self.taps.last_mut().unwrap_unchecked() })
  }
  /// Gets access to the taps, which are in the same order that they were added.
  pub fn taps(&self) -> &[DelayLineTap] {
    &self.taps
  }
  /// Gets mutable access to the taps, which are in the same order that they were added.
  pub fn taps_mut(&mut self) -> &mut[DelayLineTap] {
    &mut self.taps
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
    // Drop any DelayLineTap that refers to the DelayLine before the DelayLine is freed.
    self.taps.clear();
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
