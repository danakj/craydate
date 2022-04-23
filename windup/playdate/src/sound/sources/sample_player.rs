use core::marker::PhantomData;
use core::mem::ManuallyDrop;

use super::super::audio_sample::AudioSample;
use super::super::SoundCompletionCallback;
use super::sound_source::{SoundSource, AsSoundSource};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::{RelativeTimeSpan, TimeDelta};

#[derive(Debug)]
pub struct SamplePlayer<'sample> {
  source: ManuallyDrop<SoundSource>,
  ptr: *mut CSamplePlayer,
  loop_callback: Option<RegisteredCallback>,
  _marker: PhantomData<&'sample AudioSample>,
}
impl SamplePlayer<'_> {
  /// Creates a new SamplePlayer.
  pub fn new(sample: &AudioSample) -> Self {
    let ptr = unsafe { Self::fns().newPlayer.unwrap()() };
    unsafe { Self::fns().setSample.unwrap()(ptr, sample.cptr()) }
    SamplePlayer {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
      ptr,
      loop_callback: None,
      _marker: PhantomData,
    }
  }

  /// Returns the length of AudioSample assigned to the player.
  pub fn len(&self) -> TimeDelta {
    TimeDelta::from_seconds_lossy(unsafe { Self::fns().getLength.unwrap()(self.ptr) })
  }

  /// Starts playing the sample attached to the player.
  ///
  /// If repeat is greater than one, it loops the given number of times. If zero, it loops endlessly
  /// until it is stopped with `stop()`. If negative one, it does ping-pong looping.
  ///
  /// Sets the playback rate for the player. 1.0 is normal speed, 0.5 is down an octave, 2.0 is up
  /// an octave, etc.
  pub fn play(&mut self, repeat: i32, rate: f32) {
    // TODO: What does the return value of play() mean here?
    unsafe { Self::fns().play.unwrap()(self.ptr, repeat, rate) };
  }
  pub fn stop(&mut self) {
    unsafe { Self::fns().stop.unwrap()(self.ptr) };
  }
  /// Pauses playback of the SamplePlayer.
  pub fn pause(&mut self) {
    unsafe { Self::fns().setPaused.unwrap()(self.ptr, 1) }
  }
  /// Resumes playback of the SamplePlayer.
  pub fn unpause(&mut self) {
    unsafe { Self::fns().setPaused.unwrap()(self.ptr, 1) }
  }
  /// Returns if the player is playing a sample.
  pub fn is_playing(&self) -> bool {
    unsafe { Self::fns().isPlaying.unwrap()(self.ptr) != 0 }
  }

  /// Sets the current offset of the SamplePlayer.
  pub fn set_offset(&mut self, offset: TimeDelta) {
    unsafe { Self::fns().setOffset.unwrap()(self.ptr, offset.to_seconds()) };
  }
  /// Gets the current offset of the SamplePlayer.
  pub fn offset(&mut self) -> TimeDelta {
    TimeDelta::from_seconds_lossy(unsafe { Self::fns().getOffset.unwrap()(self.ptr) })
  }

  /// Sets the ping-pong range when `play()` is called with `repeat` of `-1`.
  pub fn set_play_range(&mut self, play_range: RelativeTimeSpan) {
    unsafe {
      Self::fns().setPlayRange.unwrap()(
        self.ptr,
        play_range.start.to_sample_frames(),
        play_range.end.to_sample_frames(),
      )
    };
  }

  /// Sets the playback rate for the SamplePlayer.
  ///
  /// 1.0 is normal speed, 0.5 is down an octave, 2.0 is up an octave, etc.
  pub fn set_rate(&mut self, rate: f32) {
    unsafe { Self::fns().setRate.unwrap()(self.ptr, rate) }
  }
  /// Gets the playback rate for the SamplePlayer.
  pub fn rate(&self) -> f32 {
    unsafe { Self::fns().getRate.unwrap()(self.ptr) }
  }

  /// Sets a function to be called every time the sample loops.
  pub fn set_loop_callback<'a, T, F: Fn(T) + 'static>(
    &mut self,
    loop_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = loop_callback.into_inner().and_then(|(callbacks, cb)| {
      // This pointer is not aligned, but we will not deref it. It's only used as a map key.
      let key = unsafe { self.as_source_mut().cptr().add(1) } as usize;
      let (func, reg) = callbacks.add_sound_source_completion(key, cb);
      self.loop_callback = Some(reg);
      Some(func)
    });
    unsafe { Self::fns().setLoopCallback.unwrap()(self.ptr, func) }
  }

  fn fns() -> &'static playdate_sys::playdate_sound_sampleplayer {
    unsafe { &*CApiState::get().csound.sampleplayer }
  }
}
impl Drop for SamplePlayer<'_> {
  fn drop(&mut self) {
    self.set_loop_callback(SoundCompletionCallback::none());
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freePlayer.unwrap()(self.ptr) }
  }
}
impl AsRef<SoundSource> for SamplePlayer<'_> {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for SamplePlayer<'_> {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}
