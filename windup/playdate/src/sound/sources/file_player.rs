use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::loop_sound_span::LoopTimeSpan;
use super::super::{SoundCompletionCallback, StereoVolume};
use super::sound_source::{AsSoundSource, SoundSource};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;
use crate::time::{TimeDelta, TimeTicks};

/// FilePlayer is used for streaming audio from a file on disk.
///
/// This requires less memory than keeping all of the file’s data in memory (as with the
/// SamplePlayer), but can increase overhead at run time.
///
/// FilePlayer can play MP3 files, but MP3 decoding is CPU-intensive. For a balance of good
/// performance and small file size, we recommend encoding audio into ADPCM .wav files.
///
/// # Preparing your sound files
/// To encode into ADPCM with Audacity
/// * File > Export Audio… > File type: WAV (Microsoft), Encoding: IMA ADPCM.
///
/// To encode into ADPCM with ffmpeg
/// * type `ffmpeg -i input.mp3 -acodec adpcm_ima_wav output.wav` at the command line.
pub struct FilePlayer {
  source: ManuallyDrop<SoundSource>,
  ptr: NonNull<CFilePlayer>,
  fade_callback: Option<RegisteredCallback>,
}
impl FilePlayer {
  /// Prepares the player to steam the file at `path`.
  ///
  /// Returns `Error::NotFoundError` if the file was not found or could not be loaded.
  pub fn from_file(path: &str) -> Result<Self, Error> {
    let ptr = unsafe { Self::fns().newPlayer.unwrap()() };
    let r =
      unsafe { Self::fns().loadIntoPlayer.unwrap()(ptr, path.to_null_terminated_utf8().as_ptr()) };
    if r == 0 {
      Err(Error::NotFoundError)
    } else {
      Ok(FilePlayer {
        source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
        ptr: NonNull::new(ptr).unwrap(),
        fade_callback: None,
      })
    }
  }

  /// Returns the length, in seconds, of the file loaded into player.
  pub fn file_len(&self) -> TimeTicks {
    // getLength() takes a mutable pointer it changes no visible state.
    let f = unsafe { Self::fns().getLength.unwrap()(self.cptr() as *mut _) };
    TimeTicks::from_seconds_lossy(f)
  }

  /// Sets the length of the buffer which will be filled from the file.
  pub fn set_buffer_len(&mut self, length: TimeTicks) {
    unsafe { Self::fns().setBufferLength.unwrap()(self.cptr_mut(), length.to_seconds()) };
  }

  /// Pauses the file player.
  pub fn pause(&mut self) {
    unsafe { Self::fns().pause.unwrap()(self.cptr_mut()) }
  }
  /// Starts playing the file player.
  ///
  /// If `times` is greater than one, it loops the given number of times. If zero, it loops
  /// endlessly until it is stopped with `stop()`.
  ///
  /// The FilePlayer lazily opens the file when it needs to, which means it's possible for the
  /// `FilePlayer` to be constructed successfully from a file, but then fail to `play()` when it
  /// tries to open and read from the file. In that case, an error is returned.
  pub fn play(&mut self, times: i32) -> Result<(), Error> {
    match unsafe { Self::fns().play.unwrap()(self.cptr_mut(), times) } {
      0 => Err(Error::PlayFileError),
      _ => Ok(()),
    }
  }
  /// Stops playing the file.
  pub fn stop(&mut self) {
    unsafe { Self::fns().stop.unwrap()(self.cptr_mut()) }
  }
  /// Returns whether the player has underrun.
  pub fn did_underrun(&self) -> bool {
    // didUnderrun() takes a mutable pointer it changes no visible state.
    unsafe { Self::fns().didUnderrun.unwrap()(self.cptr() as *mut _) != 0 }
  }
  /// Sets the start and end of the loop region for playback.
  ///
  /// If `end` is `None`, the end of the player's buffer is used.
  pub fn set_loop_range(&mut self, loop_range: LoopTimeSpan) {
    unsafe {
      Self::fns().setLoopRange.unwrap()(
        self.cptr_mut(),
        loop_range.start().to_seconds(),
        loop_range.end().map_or(0f32, TimeTicks::to_seconds),
      )
    }
  }
  /// Sets the current offset for the player.
  pub fn set_offset(&mut self, offset: TimeTicks) {
    unsafe { Self::fns().setOffset.unwrap()(self.cptr_mut(), offset.to_seconds()) }
  }
  /// Gets the current offset for the player.
  pub fn offset(&self) -> TimeTicks {
    // getOffset() takes a mutable pointer it changes no visible state.
    TimeTicks::from_seconds_lossy(unsafe { Self::fns().getOffset.unwrap()(self.cptr() as *mut _) })
  }
  /// Sets the playback rate for the player.
  ///
  /// 1.0 is normal speed, 0.5 is down an octave, 2.0 is up an octave, etc. Unlike sampleplayers,
  /// fileplayers can’t play in reverse (i.e., rate < 0).
  pub fn set_playback_rate(&mut self, rate: f32) {
    unsafe { Self::fns().setRate.unwrap()(self.cptr_mut(), rate) }
  }
  /// Gets the playback rate for the player.
  pub fn playback_rate(&self) -> f32 {
    // getRate() takes a mutable pointer it changes no visible state.
    unsafe { Self::fns().getRate.unwrap()(self.cptr() as *mut _) }
  }
  /// If flag evaluates to true, the player will restart playback (after an audible stutter) as soon
  /// as data is available.
  pub fn set_stop_on_underrun(&mut self, stop: bool) {
    unsafe { Self::fns().setStopOnUnderrun.unwrap()(self.cptr_mut(), stop as i32) }
  }
  /// Changes the volume of the fileplayer to `volume` over a length of `duration`.
  ///
  /// The callback, if not `SoundCompletionCallback::none()`, will be registered as a system event,
  /// and the application will be notified to run the callback via a `SystemEvent::Callback` event.
  /// When that occurs, the application's `Callbacks` object which was used to construct the
  /// `completion_callback` can be `run()` to execute the closure bound in the
  /// `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// player.fade_volume(
  ///   vol,
  ///   TimeDelta::from_seconds(2),
  ///   SoundCompletionCallback::with(&mut callbacks).call(|i: i32| {
  ///     println("fade done");
  ///   })
  /// );
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.runs();
  ///   }
  /// }
  /// ```
  pub fn fade_volume<'a, T, F: Fn(T) + 'static>(
    &mut self,
    volume: StereoVolume,
    duration: TimeDelta,
    completion_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = completion_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.as_source_mut().cptr() as usize;
      let (func, reg) = callbacks.add_sound_source_completion(key, cb);
      self.fade_callback = Some(reg);
      Some(func)
    });
    unsafe {
      Self::fns().fadeVolume.unwrap()(
        self.cptr_mut(),
        volume.left.into(),
        volume.right.into(),
        duration.to_sample_frames(),
        func,
      )
    }
  }

  pub(crate) fn cptr(&self) -> *const CFilePlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CFilePlayer {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_fileplayer {
    unsafe { &*CApiState::get().csound.fileplayer }
  }
}
impl Drop for FilePlayer {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freePlayer.unwrap()(self.cptr_mut()) };
  }
}

impl AsRef<SoundSource> for FilePlayer {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for FilePlayer {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}
