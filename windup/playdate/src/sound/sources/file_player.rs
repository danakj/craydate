use core::mem::ManuallyDrop;

use super::super::loop_sound_span::LoopTimeSpan;
use super::super::{SoundCompletionCallback, StereoVolume};
use super::sound_source::SoundSource;
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
  ptr: *mut CFilePlayer,
  fade_callback: Option<RegisteredCallback>,
}
impl FilePlayer {
  /// Prepares the player to steam the file at `path`.
  pub fn from_file(path: &str) -> Self {
    let ptr = unsafe { (*CApiState::get().csound.fileplayer).newPlayer.unwrap()() };
    unsafe {
      (*CApiState::get().csound.fileplayer).loadIntoPlayer.unwrap()(
        ptr,
        path.to_null_terminated_utf8().as_ptr(),
      )
    }
    // TODO: If file loading fails, file_length() would return -1 in the future:
    // https://devforum.play.date/t/playing-sounds-using-c-api/4228/3, and we should surface errors
    // somehow.
    FilePlayer {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
      ptr,
      fade_callback: None,
    }
  }
  fn as_ptr(&self) -> *const CFilePlayer {
    self.source.cptr() as *const CFilePlayer
  }
  fn as_mut_ptr(&mut self) -> *mut CFilePlayer {
    self.source.cptr() as *mut CFilePlayer
  }
  pub fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
  }

  /// Returns the length, in seconds, of the file loaded into player.
  pub fn file_len(&self) -> TimeTicks {
    let f = unsafe {
      (*CApiState::get().csound.fileplayer).getLength.unwrap()(self.as_ptr() as *mut CFilePlayer)
    };
    TimeTicks::from_seconds_lossy(f)
  }

  /// Sets the length of the buffer which will be filled from the file.
  pub fn set_buffer_len(&mut self, length: TimeTicks) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setBufferLength.unwrap()(
        self.as_mut_ptr(),
        length.to_seconds(),
      )
    };
  }

  /// Pauses the file player.
  pub fn pause(&mut self) {
    unsafe { (*CApiState::get().csound.fileplayer).pause.unwrap()(self.as_mut_ptr()) }
  }
  /// Starts playing the file player.
  ///
  /// If `times` is greater than one, it loops the given number of times. If zero, it loops
  /// endlessly until it is stopped with `stop()`.
  pub fn play(&mut self, times: i32) -> Result<(), Error> {
    // TODO: Return play()'s int output value? What is it?
    match unsafe { (*CApiState::get().csound.fileplayer).play.unwrap()(self.as_mut_ptr(), times) } {
      0 => Err("FilePlayer error on play".into()),
      _ => Ok(()),
    }
  }
  /// Stops playing the file.
  pub fn stop(&mut self) {
    unsafe { (*CApiState::get().csound.fileplayer).stop.unwrap()(self.as_mut_ptr()) }
  }
  /// Returns whether the player has underrun.
  pub fn did_underrun(&self) -> bool {
    unsafe {
      (*CApiState::get().csound.fileplayer).didUnderrun.unwrap()(self.as_ptr() as *mut CFilePlayer)
        != 0
    }
  }
  /// Sets the start and end of the loop region for playback.
  ///
  /// If `end` is `None`, the end of the player's buffer is used.
  pub fn set_loop_range(&mut self, loop_range: LoopTimeSpan) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setLoopRange.unwrap()(
        self.as_mut_ptr(),
        loop_range.start().to_seconds(),
        loop_range.end().map_or(0f32, TimeTicks::to_seconds),
      )
    }
  }
  /// Sets the current offset for the player.
  pub fn set_offset(&mut self, offset: TimeTicks) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setOffset.unwrap()(
        self.as_mut_ptr(),
        offset.to_seconds(),
      )
    }
  }
  /// Gets the current offset for the player.
  pub fn offset(&self) -> TimeTicks {
    TimeTicks::from_seconds_lossy(unsafe {
      (*CApiState::get().csound.fileplayer).getOffset.unwrap()(self.as_ptr() as *mut CFilePlayer)
    })
  }
  /// Sets the playback rate for the player.
  ///
  /// 1.0 is normal speed, 0.5 is down an octave, 2.0 is up an octave, etc. Unlike sampleplayers,
  /// fileplayers can’t play in reverse (i.e., rate < 0).
  pub fn set_playback_rate(&mut self, rate: f32) {
    unsafe { (*CApiState::get().csound.fileplayer).setRate.unwrap()(self.as_mut_ptr(), rate) }
  }
  /// Gets the playback rate for the player.
  pub fn playback_rate(&self) -> f32 {
    unsafe {
      (*CApiState::get().csound.fileplayer).getRate.unwrap()(self.as_ptr() as *mut CFilePlayer)
    }
  }
  /// If flag evaluates to true, the player will restart playback (after an audible stutter) as soon
  /// as data is available.
  pub fn set_stop_on_underrun(&mut self, stop: bool) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setStopOnUnderrun.unwrap()(
        self.as_mut_ptr(),
        stop as i32,
      )
    }
  }
  /// Changes the volume of the fileplayer to `volume` over a length of `duration`.
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
      (*CApiState::get().csound.fileplayer).fadeVolume.unwrap()(
        self.as_mut_ptr(),
        volume.left.into(),
        volume.right.into(),
        duration.to_sample_frames(),
        func,
      )
    }
  }
}
impl Drop for FilePlayer {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { (*CApiState::get().csound.fileplayer).freePlayer.unwrap()(self.ptr) };
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
