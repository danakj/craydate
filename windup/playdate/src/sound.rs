use alloc::rc::{Rc, Weak};
use core::mem::ManuallyDrop;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::*;

#[derive(Debug)]
pub struct Sound {
  state: &'static CApiState,
  default_channel: SoundChannelRef,
}
impl Sound {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Sound {
      state,
      default_channel: SoundChannelRef {
        ptr: Rc::new(unsafe { state.csound.getDefaultChannel.unwrap()() }),
        state,
      },
    }
  }

  pub fn default_channel(&self) -> &SoundChannelRef {
    &self.default_channel
  }
  pub fn default_channel_mut(&mut self) -> &mut SoundChannelRef {
    &mut self.default_channel
  }

  pub fn new_channel(&self) -> SoundChannel {
    SoundChannel {
      cref: SoundChannelRef {
        ptr: Rc::new(unsafe { (*self.state.csound.channel).newChannel.unwrap()() }),
        state: self.state,
      },
      added: false,
    }
  }
  pub fn new_fileplayer(&self) -> FilePlayer {
    FilePlayer::new(self.state)
  }

  pub fn add_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(true);
    unsafe { self.state.csound.addChannel.unwrap()(*channel.cref.ptr) };
  }
  pub fn remove_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(false);
    unsafe { self.state.csound.removeChannel.unwrap()(*channel.cref.ptr) }
  }

  /// Returns the sound engine’s current time value, in units of sample frames, 44,100 per second.
  pub fn current_sound_time(&self) -> SampleFrames {
    SampleFrames(unsafe { self.state.csound.getCurrentTime.unwrap()() })
  }

  /// Force audio output to the given outputs, regardless of headphone status.
  pub fn set_active_outputs(&self, headphone: bool, speaker: bool) {
    unsafe { self.state.csound.setOutputsActive.unwrap()(headphone as i32, speaker as i32) };
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
#[derive(Debug)]
pub struct SoundChannelRef {
  state: &'static CApiState,
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
    unsafe { (*self.state.csound.channel).getVolume.unwrap()(*self.ptr) }
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
      unsafe { self.state.csound.removeChannel.unwrap()(*self.ptr) }
    }
    unsafe { (*self.state.csound.channel).freeChannel.unwrap()(*self.ptr) }
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
  state: &'static CApiState,
  ptr: *mut CSoundSource,
  // The `channel` is set when the SoundSource has been added to the SoundChannel.
  channel: Option<Weak<*mut CSoundChannel>>, // Don't hold a borrow on SoundChannel.
}

impl SoundSource {
  /// Attach the SoundSource to the `channel` if it is not already attached to a channel.
  fn attach_to_channel(&mut self, channel: Weak<*mut CSoundChannel>) {
    // Mimic the Playdate API behaviour. Attaching a Source to a Channel when it's already attached
    // does nothing.
    if self.channel.is_none() {
      let rc_ptr = unsafe { channel.upgrade().unwrap_unchecked() };
      unsafe { (*self.state.csound.channel).addSource.unwrap()(*rc_ptr, self.ptr) };
      self.channel = Some(channel);
    }
  }

  /// Removes the SoundSource from the `channel` if it was currently attached.
  ///
  /// If the SoundSource is not attached to `channel`, then `Error::NotFoundError` is returned.
  fn detach_from_channel(&mut self, channel: Rc<*mut CSoundChannel>) -> Result<(), Error> {
    if let Some(attached_channel) = &mut self.channel {
      if attached_channel.ptr_eq(&Rc::downgrade(&channel)) {
        let r = unsafe { (*self.state.csound.channel).removeSource.unwrap()(*channel, self.ptr) };
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
    unsafe { (*self.state.csound.source).getVolume.unwrap()(self.ptr, &mut v.left, &mut v.right) };
    v
  }
  /// Sets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn set_volume(&mut self, v: SoundSourceVolume) {
    unsafe {
      (*self.state.csound.source).setVolume.unwrap()(
        self.ptr,
        v.left.clamp(0f32, 1f32),
        v.right.clamp(0f32, 1f32),
      )
    }
  }
  /// Returns whether the source is currently playing.
  pub fn is_playing(&self) -> bool {
    unsafe { (*self.state.csound.source).isPlaying.unwrap()(self.ptr) != 0 }
  }
}

impl Drop for SoundSource {
  fn drop(&mut self) {
    if let Some(weak_ptr) = self.channel.take() {
      if let Some(rc_ptr) = weak_ptr.upgrade() {
        let r = self.detach_from_channel(rc_ptr);
        assert!(r.is_ok()); // Otherwise, `self.channel` was lying.
      }
    }
  }
}

pub struct FilePlayer {
  source: ManuallyDrop<SoundSource>,
  ptr: *mut CFilePlayer,
}
impl FilePlayer {
  fn new(state: &'static CApiState) -> Self {
    let ptr = unsafe { (*state.csound.fileplayer).newPlayer.unwrap()() };
    FilePlayer {
      source: ManuallyDrop::new(SoundSource {
        state,
        ptr: ptr as *mut CSoundSource,
        channel: None,
      }),
      ptr,
    }
  }
  fn as_ptr(&self) -> *mut CFilePlayer {
    self.source.ptr as *mut CFilePlayer
  }
  pub fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
  }

  // TODO: setFinishCallback

  /// Prepares the player to steam the file at `path`.
  pub fn load_file(&mut self, path: &str) {
    unsafe {
      (*self.source.state.csound.fileplayer).loadIntoPlayer.unwrap()(
        self.as_ptr(),
        path.to_null_terminated_utf8().as_ptr(),
      )
    }
  }

  /// Returns the length, in seconds, of the file loaded into player.
  pub fn len(&self) -> TimeTicks {
    let f = unsafe { (*self.source.state.csound.fileplayer).getLength.unwrap()(self.as_ptr()) };
    TimeTicks::from_seconds_lossy(f)
  }
  /// Sets the buffer length of the player to the given length.
  pub fn set_len(&mut self, time: TimeTicks) {
    unsafe {
      (*self.source.state.csound.fileplayer).setBufferLength.unwrap()(
        self.as_ptr(),
        time.to_seconds(),
      )
    };
  }

  /// Pauses the file player.
  pub fn pause(&mut self) {
    unsafe { (*self.source.state.csound.fileplayer).pause.unwrap()(self.as_ptr()) }
  }
  /// Starts playing the file player.
  ///
  /// If repeat is greater than one, it loops the given number of times. If zero, it loops endlessly
  /// until it is stopped with `stop()`.
  pub fn play(&mut self, repeat: i32) {
    // TODO: Return play()'s int output value? What is it?
    unsafe { (*self.source.state.csound.fileplayer).play.unwrap()(self.as_ptr(), repeat) };
  }
  /// Stops playing the file.
  pub fn stop(&mut self) {
    unsafe { (*self.source.state.csound.fileplayer).stop.unwrap()(self.as_ptr()) }
  }
  /// Returns whether the player has underrun.
  pub fn did_underrun(&self) -> bool {
    unsafe { (*self.source.state.csound.fileplayer).didUnderrun.unwrap()(self.as_ptr()) != 0 }
  }
  /// Sets the start and end of the loop region for playback.
  ///
  /// If `end` is `None`, the end of the player's buffer is used.
  pub fn set_loop_range(&mut self, start: TimeTicks, end: Option<TimeTicks>) {
    unsafe {
      (*self.source.state.csound.fileplayer).setLoopRange.unwrap()(
        self.as_ptr(),
        start.to_seconds(),
        end.map_or(0f32, TimeTicks::to_seconds),
      )
    }
  }
  /// Sets the current offset for the player.
  pub fn set_offset(&mut self, offset: TimeTicks) {
    unsafe {
      (*self.source.state.csound.fileplayer).setOffset.unwrap()(self.as_ptr(), offset.to_seconds())
    }
  }
  /// Gets the current offset for the player.
  pub fn offset(&self) -> TimeTicks {
    TimeTicks::from_seconds_lossy(unsafe {
      (*self.source.state.csound.fileplayer).getOffset.unwrap()(self.as_ptr())
    })
  }
  /// Sets the playback rate for the player.
  ///
  /// 1.0 is normal speed, 0.5 is down an octave, 2.0 is up an octave, etc. Unlike sampleplayers,
  /// fileplayers can’t play in reverse (i.e., rate < 0).
  pub fn set_playback_rate(&mut self, rate: f32) {
    unsafe { (*self.source.state.csound.fileplayer).setRate.unwrap()(self.as_ptr(), rate) }
  }
  /// Gets the playback rate for the player.
  pub fn playback_rate(&self) -> f32 {
    unsafe { (*self.source.state.csound.fileplayer).getRate.unwrap()(self.as_ptr()) }
  }
  /// If flag evaluates to true, the player will restart playback (after an audible stutter) as soon
  /// as data is available.
  pub fn set_stop_on_underrun(&mut self, stop: bool) {
    unsafe {
      (*self.source.state.csound.fileplayer).setStopOnUnderrun.unwrap()(self.as_ptr(), stop as i32)
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
      (*self.source.state.csound.fileplayer).fadeVolume.unwrap()(
        self.as_ptr(),
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
    unsafe { (*self.source.state.csound.fileplayer).freePlayer.unwrap()(self.ptr) };
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
