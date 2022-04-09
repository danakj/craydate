use alloc::rc::{Rc, Weak};
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::*;

const SAMPLE_FRAMES_PER_SEC: i32 = 44_100;

pub type SoundCompletionCallback<'a, T, F, S> =
  crate::callbacks::CallbackBuilder<'a, T, F, AllowNull, S>;

/// Returns whether a SoundFormat is stereo. Otherwise, it is mono.
pub fn sound_format_is_stereo(sound_format: SoundFormat) -> bool {
  sound_format.0 & 1 == 1
}

/// Returns whether a SoundFormat is 16 bit. Otherwise, it is 8 bit.
pub fn sound_format_is_16_bit(sound_format: SoundFormat) -> bool {
  sound_format.0 >= SoundFormat::kSound16bitMono.0
}

/// Returns the number of bytes per sample frame for the SoundFormat.
pub fn sound_format_bytes_per_frame(sound_format: SoundFormat) -> usize {
  let stereo = if sound_format_is_stereo(sound_format) {
    2
  } else {
    1
  };
  let bytes = if sound_format_is_16_bit(sound_format) {
    2
  } else {
    1
  };
  stereo * bytes
}

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

  fn set_added(&mut self, added: bool) {
    self.added = added
  }
}

#[derive(Debug)]
pub struct SoundChannelRef {
  // This class holds an Rc but is not Clone. This allows it to know when the Rc is going away, in
  // order to clean up other related stuff.
  ptr: Rc<*mut CSoundChannel>,
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

#[derive(Debug, Default)]
pub struct StereoVolume {
  pub left: f32,  // TODO: Replace with some Type<f32> that clamps the value to 0-1.
  pub right: f32, // TODO: Replace with some Type<f32> that clamps the value to 0-1.
}
impl StereoVolume {
  pub fn new(left: f32, right: f32) -> Self {
    StereoVolume { left, right }
  }
  pub fn zero() -> Self {
    Self::new(0.0, 0.0)
  }
  pub fn one() -> Self {
    Self::new(1.0, 1.0)
  }
}

#[derive(Debug)]
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
  pub fn volume(&self) -> StereoVolume {
    let mut v = StereoVolume {
      left: 0.0,
      right: 0.0,
    };
    unsafe {
      (*CApiState::get().csound.source).getVolume.unwrap()(self.ptr, &mut v.left, &mut v.right)
    };
    v
  }
  /// Sets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn set_volume(&mut self, v: StereoVolume) {
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

  pub fn set_completion_callback<'a, T, F: Fn(T) + 'static>(
    &mut self,
    completion_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = completion_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.ptr as usize;
      let (func, reg) = callbacks.add_sound_source_completion(key, cb);
      self.completion_callback = Some(reg);
      Some(func)
    });
    unsafe { (*CApiState::get().csound.source).setFinishCallback.unwrap()(self.ptr, func) }
  }
}

impl Drop for SoundSource {
  fn drop(&mut self) {
    self.set_completion_callback(SoundCompletionCallback::none());

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
      source: ManuallyDrop::new(SoundSource::new(ptr as *mut CSoundSource)),
      ptr,
      fade_callback: None,
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
  /// Changes the volume of the fileplayer to `volume` over a length of `duration`.
  pub fn fade_volume<'a, T, F: Fn(T) + 'static>(
    &mut self,
    volume: StereoVolume,
    duration: TimeDelta,
    completion_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = completion_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.as_source_mut().ptr as usize;
      let (func, reg) = callbacks.add_sound_source_completion(key, cb);
      self.fade_callback = Some(reg);
      Some(func)
    });
    let num_samples = duration.total_whole_milliseconds() * SAMPLE_FRAMES_PER_SEC / 1000;
    unsafe {
      (*CApiState::get().csound.fileplayer).fadeVolume.unwrap()(
        self.as_mut_ptr(),
        volume.left.clamp(0f32, 1f32),
        volume.right.clamp(0f32, 1f32),
        num_samples,
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

#[derive(Debug)]
pub struct SamplePlayer<'sample, 'data> {
  source: ManuallyDrop<SoundSource>,
  ptr: *mut CSamplePlayer,
  loop_callback: Option<RegisteredCallback>,
  _marker: PhantomData<&'sample AudioSample<'data>>,
}
impl<'data> SamplePlayer<'_, 'data> {
  pub fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
  }

  /// Creates a new SamplePlayer.
  pub fn new(sample: &AudioSample<'data>) -> Self {
    let ptr = unsafe { Self::fns().newPlayer.unwrap()() };
    unsafe { Self::fns().setSample.unwrap()(ptr, sample.ptr) }
    SamplePlayer {
      source: ManuallyDrop::new(SoundSource::new(ptr as *mut CSoundSource)),
      ptr,
      loop_callback: None,
      _marker: PhantomData,
    }
  }

  /// Returns the length of AudioSample assigned to the player.
  pub fn len(&self) -> TimeTicks {
    TimeTicks::from_seconds_lossy(unsafe { Self::fns().getLength.unwrap()(self.ptr) })
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
  pub fn set_play_range(&mut self, start: TimeDelta, end: TimeDelta) {
    let start_frame = start.total_whole_milliseconds() * SAMPLE_FRAMES_PER_SEC / 1000;
    let end_frame = end.total_whole_milliseconds() * SAMPLE_FRAMES_PER_SEC / 1000;
    unsafe { Self::fns().setPlayRange.unwrap()(self.ptr, start_frame, end_frame) };
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

  /// Sets the playback volume for left and right channels.
  pub fn set_volume(&mut self, volume: StereoVolume) {
    unsafe { Self::fns().setVolume.unwrap()(self.ptr, volume.left, volume.right) }
  }
  /// Gets the current left and right channel volume of the SamplePlayer.
  pub fn volume(&self) -> StereoVolume {
    let mut volume = StereoVolume::default();
    unsafe { Self::fns().getVolume.unwrap()(self.ptr, &mut volume.left, &mut volume.right) };
    volume
  }

  /// Sets a function to be called every time the sample loops.
  pub fn set_loop_callback<'a, T, F: Fn(T) + 'static>(
    &mut self,
    loop_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = loop_callback.into_inner().and_then(|(callbacks, cb)| {
      // This pointer is not aligned, but we will not deref it. It's only used as a map key.
      let key = unsafe { self.as_source_mut().ptr.add(1) } as usize;
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
impl Drop for SamplePlayer<'_, '_> {
  fn drop(&mut self) {
    self.set_loop_callback(SoundCompletionCallback::none());
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freePlayer.unwrap()(self.ptr) }
  }
}
impl AsRef<SoundSource> for SamplePlayer<'_, '_> {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for SamplePlayer<'_, '_> {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}

pub struct AudioSample<'data> {
  ptr: *mut CAudioSample,
  _marker: PhantomData<&'data u8>,
}
impl<'data> AudioSample<'data> {
  fn from_ptr<'a>(ptr: *mut CAudioSample) -> AudioSample<'a> {
    AudioSample {
      ptr,
      _marker: PhantomData,
    }
  }

  /// Creates a new AudioSample with a buffer large enough to load a file of length
  /// `bytes`.
  pub fn with_bytes(bytes: usize) -> Self {
    let ptr = unsafe { (*CApiState::get().csound.sample).newSampleBuffer.unwrap()(bytes as i32) };
    Self::from_ptr(ptr)
  }

  /// Creates a new AudioSample, with the sound data loaded in memory. If there is no file at path,
  /// the function returns None.
  pub fn from_file<'a>(path: &str) -> Option<AudioSample<'a>> {
    let ptr = unsafe {
      (*CApiState::get().csound.sample).load.unwrap()(path.to_null_terminated_utf8().as_ptr())
    };
    if ptr.is_null() {
      None
    } else {
      Some(Self::from_ptr(ptr))
    }
  }

  /// Creates a new AudioSample referencing the given audio data.
  ///
  /// The AudioSample keeps a pointer to the data instead of copying it.
  pub fn from_data<'a>(data: &'a [u8], format: SoundFormat, sample_rate: u32) -> AudioSample<'a> {
    assert!(
      format == SoundFormat::kSound8bitMono
        || format == SoundFormat::kSound8bitStereo
        || format == SoundFormat::kSound16bitMono
        || format == SoundFormat::kSound16bitStereo
        || format == SoundFormat::kSoundADPCMMono
        || format == SoundFormat::kSound16bitStereo
    );
    let ptr = unsafe {
      (*CApiState::get().csound.sample).newSampleFromData.unwrap()(
        data.as_ptr() as *mut u8, // the CAudioSample holds a reference to the `data`.
        format,
        sample_rate,
        data.len() as i32,
      )
    };
    Self::from_ptr(ptr)
  }

  /// Loads the sound data from the file at `path` into the existing AudioSample.
  pub fn load_file(&mut self, path: &str) {
    unsafe {
      (*CApiState::get().csound.sample).loadIntoSample.unwrap()(
        self.ptr,
        path.to_null_terminated_utf8().as_ptr(),
      )
    };
  }

  /// Returns the length of the AudioSample.
  pub fn len(&self) -> TimeTicks {
    TimeTicks::from_seconds_lossy(unsafe {
      (*CApiState::get().csound.sample).getLength.unwrap()(self.ptr)
    })
  }

  fn all_data(&self) -> (*mut u8, SoundFormat, u32, u32) {
    let mut ptr = MaybeUninit::uninit();
    let mut format = MaybeUninit::uninit();
    let mut sample_rate = MaybeUninit::uninit();
    let mut bytes = MaybeUninit::uninit();
    unsafe {
      (*CApiState::get().csound.sample).getData.unwrap()(
        self.ptr,
        ptr.as_mut_ptr(),
        format.as_mut_ptr(),
        sample_rate.as_mut_ptr(),
        bytes.as_mut_ptr(),
      )
    };
    unsafe {
      (
        ptr.assume_init(),
        format.assume_init(),
        sample_rate.assume_init(),
        bytes.assume_init(),
      )
    }
  }

  /// Retrieves the sample’s data.
  // Note: No mutable access to the buffer is provided for 2 reasons:
  // 1) The from_data() constructor allows the caller to keep a shared reference on the data, so we
  //    must not make an aliased mutable reference. We could instead own the data in this struct,
  //    but...
  // 2) Audio runs on a different thread, so changing data in the AudioSample is probably not
  //    intended and would be a data race.
  pub fn data(&self) -> &'data [u8] {
    let (ptr, _, _, bytes) = self.all_data();
    unsafe { core::slice::from_raw_parts(ptr, bytes as usize) }
  }

  /// Retrieves the sample’s SoundFormat.
  pub fn sound_format(&self) -> SoundFormat {
    let (_, format, _, _) = self.all_data();
    format
  }
  /// Retrieves the sample’s SoundFormat.
  pub fn sample_rate(&self) -> u32 {
    let (_, _, sample_rate, _) = self.all_data();
    sample_rate
  }
}
impl Drop for AudioSample<'_> {
  fn drop(&mut self) {
    unsafe { (*CApiState::get().csound.sample).freeSample.unwrap()(self.ptr) }
  }
}
