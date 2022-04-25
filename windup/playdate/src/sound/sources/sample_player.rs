use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::audio_sample::AudioSample;
use super::super::SoundCompletionCallback;
use super::sound_source::{AsSoundSource, SoundSource};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::{RelativeTimeSpan, TimeDelta};

/// A `SamplePlayer` will play an `AudioSample`.
///
/// The `SamplePlayer` acts as a `SoundSource` so it can be connected to a `SoundChannel` to play
/// the sample to the device's audio output. The `SamplePlayer` holds a borrow on the `AudioSample`
/// rather than taking ownership.
#[derive(Debug)]
pub struct SamplePlayer<'sample> {
  source: ManuallyDrop<SoundSource>,
  ptr: NonNull<CSamplePlayer>,
  loop_callback: Option<RegisteredCallback>,
  _marker: PhantomData<&'sample AudioSample>,
}
impl SamplePlayer<'_> {
  /// Creates a new SamplePlayer.
  pub fn new(sample: &AudioSample) -> Self {
    let ptr = unsafe { Self::fns().newPlayer.unwrap()() };
    // setSample() takes a mutable sample pointer but doesn't mutate any visible state.
    unsafe { Self::fns().setSample.unwrap()(ptr, sample.cptr() as *mut _) }
    SamplePlayer {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
      ptr: NonNull::new(ptr).unwrap(),
      loop_callback: None,
      _marker: PhantomData,
    }
  }

  /// Returns the length of AudioSample assigned to the player.
  pub fn len(&self) -> TimeDelta {
    // getLength() takes a mutable pointer it changes no visible state.
    TimeDelta::from_seconds_lossy(unsafe { Self::fns().getLength.unwrap()(self.cptr() as *mut _) })
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
    unsafe { Self::fns().play.unwrap()(self.cptr_mut(), repeat, rate) };
  }
  pub fn stop(&mut self) {
    unsafe { Self::fns().stop.unwrap()(self.cptr_mut()) };
  }
  /// Pauses playback of the SamplePlayer.
  pub fn pause(&mut self) {
    unsafe { Self::fns().setPaused.unwrap()(self.cptr_mut(), 1) }
  }
  /// Resumes playback of the SamplePlayer.
  pub fn unpause(&mut self) {
    unsafe { Self::fns().setPaused.unwrap()(self.cptr_mut(), 1) }
  }
  /// Returns if the player is playing a sample.
  pub fn is_playing(&self) -> bool {
    // isPlaying() takes a mutable pointer it changes no visible state.
    unsafe { Self::fns().isPlaying.unwrap()(self.cptr() as *mut _) != 0 }
  }

  /// Sets the current offset of the SamplePlayer.
  pub fn set_offset(&mut self, offset: TimeDelta) {
    unsafe { Self::fns().setOffset.unwrap()(self.cptr_mut(), offset.to_seconds()) };
  }
  /// Gets the current offset of the SamplePlayer.
  pub fn offset(&mut self) -> TimeDelta {
    // getOffset() takes a mutable pointer it changes no visible state.
    TimeDelta::from_seconds_lossy(unsafe { Self::fns().getOffset.unwrap()(self.cptr() as *mut _) })
  }

  /// Sets the ping-pong range when `play()` is called with `repeat` of `-1`.
  pub fn set_play_range(&mut self, play_range: RelativeTimeSpan) {
    unsafe {
      Self::fns().setPlayRange.unwrap()(
        self.cptr_mut(),
        play_range.start.to_sample_frames(),
        play_range.end.to_sample_frames(),
      )
    };
  }

  /// Sets the playback rate for the SamplePlayer.
  ///
  /// 1.0 is normal speed, 0.5 is down an octave, 2.0 is up an octave, etc.
  pub fn set_rate(&mut self, rate: f32) {
    unsafe { Self::fns().setRate.unwrap()(self.cptr_mut(), rate) }
  }
  /// Gets the playback rate for the SamplePlayer.
  pub fn rate(&self) -> f32 {
    // getRate() takes a mutable pointer it changes no visible state.
    unsafe { Self::fns().getRate.unwrap()(self.cptr() as *mut _) }
  }

  /// Sets a function to be called every time the sample loops.
  ///
  /// The callback will be registered as a system event, and the application will be notified to run
  /// the callback via a `SystemEvent::Callback` event. When that occurs, the application's
  /// `Callbacks` object which was used to construct the `completion_callback` can be `run()` to
  /// execute the closure bound in the `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// player.set_loop_callback(SoundCompletionCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("looped");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.runs();
  ///   }
  /// }
  /// ```
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
    unsafe { Self::fns().setLoopCallback.unwrap()(self.cptr_mut(), func) }
  }

  pub(crate) fn cptr(&self) -> *const CSamplePlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CSamplePlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_sampleplayer {
    unsafe { &*CApiState::get().csound.sampleplayer }
  }
}

impl Drop for SamplePlayer<'_> {
  fn drop(&mut self) {
    self.set_loop_callback(SoundCompletionCallback::none());
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freePlayer.unwrap()(self.cptr_mut()) }
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
