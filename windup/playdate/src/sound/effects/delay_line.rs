use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::sources::delay_line_tap::DelayLineTap;
use super::sound_effect::SoundEffect;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::TimeDelta;

// A `DelayLine` effect. A `DelayLine` acts as a `SoundEffect` which can be added to a
// `SoundChannel`.
#[derive(Debug)]
pub struct DelayLine {
  effect: ManuallyDrop<SoundEffect>,
  ptr: NonNull<CDelayLine>,
  taps: Vec<DelayLineTap>,
  length_in_frames: i32,
  max_tap_position_in_frames: i32,
}
impl DelayLine {
  /// Creates a new `DelayLine` effect.
  pub fn new(length: TimeDelta, stereo: bool) -> Self {
    let ptr =
      unsafe { Self::fns().newDelayLine.unwrap()(length.to_sample_frames(), stereo as i32) };
    DelayLine {
      effect: ManuallyDrop::new(SoundEffect::from_ptr(ptr as *mut CSoundEffect)),
      ptr: NonNull::new(ptr).unwrap(),
      taps: Vec::new(),
      length_in_frames: length.to_sample_frames(),
      max_tap_position_in_frames: 0,
    }
  }

  /// Adds a new tap on the DelayLine, at the given time position within the `DelayLine`.
  ///
  /// `delay` must be less than or equal to the length of the `DelayLine`.
  ///
  /// # Return
  /// If the `delay` is larger than the length of the `DelayLine`, it will not be added and the
  /// function will return `None`. Otherwise, it returns a borrow on the new `DelayLineTap`.
  pub fn add_tap(&mut self, delay: TimeDelta) -> Option<&mut DelayLineTap> {
    let max_frames = self.max_tap_position_in_frames.max(delay.to_sample_frames());
    if max_frames <= self.length_in_frames {
      let tap = DelayLineTap::new(self, delay);
      self.taps.push(tap);
      self.max_tap_position_in_frames = max_frames;
      Some(unsafe { self.taps.last_mut().unwrap_unchecked() })
    } else {
      None
    }
  }
  /// Gets access to the taps, which are in the same order that they were added.
  pub fn taps(&self) -> &[DelayLineTap] {
    &self.taps
  }
  /// Gets mutable access to the taps, which are in the same order that they were added.
  pub fn taps_mut(&mut self) -> &mut [DelayLineTap] {
    &mut self.taps
  }

  /// Changes the length of the delay line, clearing its contents.
  ///
  /// The `DelayLine` can not be shortened less than the position of any `DelayLineTap` that was
  /// added to it, and the specified length will be grown to be valid.
  pub fn set_len(&mut self, length: TimeDelta) {
    let length_in_frames = length.to_sample_frames().max(self.max_tap_position_in_frames);
    unsafe { Self::fns().setLength.unwrap()(self.cptr_mut(), length_in_frames) }
  }
  /// Returns the length of the delay line.
  pub fn len(&self) -> TimeDelta {
    TimeDelta::from_sample_frames(self.length_in_frames)
  }

  /// Sets the feedback level of the delay line.
  pub fn set_feedback(&mut self, feedback: f32) {
    unsafe { Self::fns().setFeedback.unwrap()(self.cptr_mut(), feedback) }
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut CDelayLine {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_effect_delayline {
    unsafe { &*(*CApiState::get().csound.effect).delayline }
  }
}

impl Drop for DelayLine {
  fn drop(&mut self) {
    // Drop any DelayLineTap that refers to the DelayLine before the DelayLine is freed.
    self.taps.clear();
    // Ensure the SoundEffect has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.effect) };
    unsafe { Self::fns().freeDelayLine.unwrap()(self.cptr_mut()) }
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
