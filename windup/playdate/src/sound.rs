use alloc::rc::{Rc, Weak};
use core::mem::ManuallyDrop;

use crate::capi_state::CApiState;
use crate::ctypes::*;
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

  pub fn volume(&self) -> SoundSourceVolume {
    let mut v = SoundSourceVolume {
      left: 0.0,
      right: 0.0,
    };
    unsafe { (*self.state.csound.source).getVolume.unwrap()(self.ptr, &mut v.left, &mut v.right) };
    v
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

  pub fn as_source(&self) -> &SoundSource {
      self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
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
