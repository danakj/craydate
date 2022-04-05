use alloc::rc::{Rc, Weak};
use core::mem::ManuallyDrop;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::*;

#[derive(Debug)]
pub struct Sound {
  default_channel: SoundChannelRef,
}
impl Sound {
  pub(crate) fn new() -> Self {
    Sound {
      default_channel: SoundChannelRef {
        ptr: Rc::new(unsafe { CApiState::get().csound.getDefaultChannel.unwrap()() }),
      },
    }
  }

  pub fn default_channel(&self) -> &SoundChannelRef {
    &self.default_channel
  }
  pub fn default_channel_mut(&mut self) -> &mut SoundChannelRef {
    &mut self.default_channel
  }

  pub fn add_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(true);
    unsafe { CApiState::get().csound.addChannel.unwrap()(*channel.cref.ptr) };
  }
  pub fn remove_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(false);
    unsafe { CApiState::get().csound.removeChannel.unwrap()(*channel.cref.ptr) }
  }

  /// Returns the sound engine’s current time value, in units of sample frames, 44,100 per second.
  pub fn current_sound_time(&self) -> SampleFrames {
    SampleFrames(unsafe { CApiState::get().csound.getCurrentTime.unwrap()() })
  }

  /// Force audio output to the given outputs, regardless of headphone status.
  pub fn set_active_outputs(&self, headphone: bool, speaker: bool) {
    unsafe { CApiState::get().csound.setOutputsActive.unwrap()(headphone as i32, speaker as i32) };
  }
}

/// SampleFrames is a unit of time in the sound engine, with 44,100 sample frames per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleFrames(u32);
impl SampleFrames {
  pub fn to_u32(self) -> u32 {
    self.0
  }
}

#[derive(Debug)]
pub struct SoundChannel {
  cref: SoundChannelRef,
  added: bool,
}
impl SoundChannel {
  pub fn new() -> SoundChannel {
    SoundChannel {
      cref: SoundChannelRef {
        ptr: Rc::new(unsafe { (*CApiState::get().csound.channel).newChannel.unwrap()() }),
      },
      added: false,
    }
  }
}

#[derive(Debug)]
pub struct SoundChannelRef {
  // This class holds an Rc but is not Clone. This allows it to know when the Rc is going away, in
  // order to clean up other related stuff.
  ptr: Rc<*mut CSoundChannel>,
}

impl SoundChannel {
  fn set_added(&mut self, added: bool) {
    self.added = added
  }
}

impl SoundChannelRef {
  /// Gets the volume for the channel, in the range [0-1].
  // TODO: Replace the ouput with a Type<f32> that clamps the value to 0-1.
  pub fn volume(&self) -> f32 {
    unsafe { (*CApiState::get().csound.channel).getVolume.unwrap()(*self.ptr) }
  }
  /// Sets the volume for the channel, in the range [0-1].
  // TODO: Replace the ouput with a Type<f32> that clamps the value to 0-1.
  pub fn set_volume(&mut self, volume: f32) {
    unsafe { (*CApiState::get().csound.channel).setVolume.unwrap()(*self.ptr, volume) }
  }

  pub fn attach_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) {
    source.as_mut().attach_to_channel(Rc::downgrade(&self.ptr));
  }
  pub fn detach_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) -> Result<(), Error> {
    source.as_mut().detach_from_channel(self.ptr.clone())
  }
}

impl Drop for SoundChannel {
  fn drop(&mut self) {
    if self.added {
      unsafe { CApiState::get().csound.removeChannel.unwrap()(*self.ptr) }
    }
    unsafe { (*CApiState::get().csound.channel).freeChannel.unwrap()(*self.ptr) }
  }
}

impl core::ops::Deref for SoundChannel {
  type Target = SoundChannelRef;

  fn deref(&self) -> &Self::Target {
    &self.cref
  }
}
impl core::ops::DerefMut for SoundChannel {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.cref
  }
}
impl core::borrow::Borrow<SoundChannelRef> for SoundChannel {
  fn borrow(&self) -> &SoundChannelRef {
    self // Calls Deref.
  }
}
impl AsRef<SoundChannelRef> for SoundChannel {
  fn as_ref(&self) -> &SoundChannelRef {
    self // Calls Deref.
  }
}

pub struct SoundSourceVolume {
  pub left: f32,  // TODO: Replace with some Type<f32> that clamps the value to 0-1.
  pub right: f32, // TODO: Replace with some Type<f32> that clamps the value to 0-1.
}

pub struct SoundSource {
  ptr: *mut CSoundSource,
  // The `channel` is set when the SoundSource has been added to the SoundChannel.
  channel: Option<Weak<*mut CSoundChannel>>, // Don't hold a borrow on SoundChannel.
  // When the RegisteredCallback is destroyed, the user-given closure will be destroyed as well.
  completion_callback: Option<RegisteredCallback>,
}

impl SoundSource {
  fn new(ptr: *mut CSoundSource) -> Self {
    SoundSource {
      ptr,
      channel: None,
      completion_callback: None,
    }
  }

  /// Attach the SoundSource to the `channel` if it is not already attached to a channel.
  fn attach_to_channel(&mut self, channel: Weak<*mut CSoundChannel>) {
    // Mimic the Playdate API behaviour. Attaching a Source to a Channel when it's already attached
    // does nothing.
    if self.channel.is_none() {
      // The SoundSource holds a Weak pointer to the SoundChannel so it knows whether to remove
      // itself in drop().
      let rc_ptr = unsafe { channel.upgrade().unwrap_unchecked() };
      unsafe { (*CApiState::get().csound.channel).addSource.unwrap()(*rc_ptr, self.ptr) };
      self.channel = Some(channel);
    }
  }

  /// Removes the SoundSource from the `channel` if it was currently attached.
  ///
  /// If the SoundSource is not attached to `channel`, then `Error::NotFoundError` is returned.
  fn detach_from_channel(&mut self, channel: Rc<*mut CSoundChannel>) -> Result<(), Error> {
    if let Some(attached_channel) = &mut self.channel {
      if attached_channel.ptr_eq(&Rc::downgrade(&channel)) {
        let r =
          unsafe { (*CApiState::get().csound.channel).removeSource.unwrap()(*channel, self.ptr) };
        assert!(r != 0);
        return Ok(());
      }
    }
    Err(Error::NotFoundError())
  }

  /// Gets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn volume(&self) -> SoundSourceVolume {
    let mut v = SoundSourceVolume {
      left: 0.0,
      right: 0.0,
    };
    unsafe {
      (*CApiState::get().csound.source).getVolume.unwrap()(self.ptr, &mut v.left, &mut v.right)
    };
    v
  }
  /// Sets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn set_volume(&mut self, v: SoundSourceVolume) {
    unsafe {
      (*CApiState::get().csound.source).setVolume.unwrap()(
        self.ptr,
        v.left.clamp(0f32, 1f32),
        v.right.clamp(0f32, 1f32),
      )
    }
  }
  /// Returns whether the source is currently playing.
  pub fn is_playing(&self) -> bool {
    unsafe { (*CApiState::get().csound.source).isPlaying.unwrap()(self.ptr) != 0 }
  }

  pub fn set_completion_callback<T>(
    &mut self,
    callbacks: &mut Callbacks<T>,
    cb: impl Fn(T) + 'static,
  ) {
    let (func, reg) = callbacks.add_sound_source_completion(self.ptr, cb);
    self.completion_callback = Some(reg);
    unsafe { (*CApiState::get().csound.source).setFinishCallback.unwrap()(self.ptr, Some(func)) }
  }
  pub fn clear_completion_callback(&mut self) {
    unsafe { (*CApiState::get().csound.source).setFinishCallback.unwrap()(self.ptr, None) }
    self.completion_callback = None;
  }
}

impl Drop for SoundSource {
  fn drop(&mut self) {
    self.clear_completion_callback();

    if let Some(weak_ptr) = self.channel.take() {
      if let Some(rc_ptr) = weak_ptr.upgrade() {
        let r = self.detach_from_channel(rc_ptr);
        assert!(r.is_ok()); // Otherwise, `self.channel` was lying.
      }
    }
  }
}

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
      source: ManuallyDrop::new(SoundSource::new(ptr as *mut CSoundSource)),
      ptr,
    }
  }
  fn as_ptr(&self) -> *const CFilePlayer {
    self.source.ptr as *const CFilePlayer
  }
  fn as_mut_ptr(&mut self) -> *mut CFilePlayer {
    self.source.ptr as *mut CFilePlayer
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
  pub fn set_buffer_len(&mut self, time: TimeTicks) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setBufferLength.unwrap()(
        self.as_mut_ptr(),
        time.to_seconds(),
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
  pub fn set_loop_range(&mut self, start: TimeTicks, end: Option<TimeTicks>) {
    unsafe {
      (*CApiState::get().csound.fileplayer).setLoopRange.unwrap()(
        self.as_mut_ptr(),
        start.to_seconds(),
        end.map_or(0f32, TimeTicks::to_seconds),
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
  /// Changes the volume of the fileplayer to `volume` over a length of `num_samples` sample frames.
  ///
  /// TODO: then calls the provided callback (if given).
  pub fn fade_volume(
    &mut self,
    volume: SoundSourceVolume,
    num_samples: i32, /* TODO: callback */
  ) {
    unsafe {
      (*CApiState::get().csound.fileplayer).fadeVolume.unwrap()(
        self.as_mut_ptr(),
        volume.left.clamp(0f32, 1f32),
        volume.right.clamp(0f32, 1f32),
        num_samples,
        None,
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
