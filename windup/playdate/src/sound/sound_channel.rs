use alloc::rc::Rc;

use super::sources::sound_source::SoundSource;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;

#[derive(Debug)]
pub struct SoundChannel {
  cref: SoundChannelRef,
  added: bool,
}
impl SoundChannel {
  pub fn new() -> SoundChannel {
    SoundChannel {
      cref: SoundChannelRef::from_ptr(unsafe {
        (*CApiState::get().csound.channel).newChannel.unwrap()()
      }),
      added: false,
    }
  }

  pub(crate) fn set_added(&mut self, added: bool) {
    self.added = added
  }
  pub(crate) fn cptr(&self) -> *mut CSoundChannel {
    *self.ptr
  }
}

impl Drop for SoundChannel {
  fn drop(&mut self) {
    if self.added {
      unsafe { CApiState::get().csound.removeChannel.unwrap()(self.cptr()) }
    }
    unsafe { (*CApiState::get().csound.channel).freeChannel.unwrap()(self.cptr()) }
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

#[derive(Debug)]
pub struct SoundChannelRef {
  // This class holds an Rc but is not Clone. This allows it to know when the Rc is going away, in
  // order to clean up other related stuff.
  ptr: Rc<*mut CSoundChannel>,
}
impl SoundChannelRef {
  pub(crate) fn from_ptr(ptr: *mut CSoundChannel) -> Self {
    SoundChannelRef { ptr: Rc::new(ptr) }
  }

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

  /// Attach the `source` to this channel.
  ///
  /// # Return
  /// Returns `Error::AlreadyAttachedError` if the `source` is already attached to a channel or (for
  /// a Synth) to an Instrument.
  pub fn attach_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) -> Result<(), Error> {
    source.as_mut().attach_to_channel(&self.ptr)
  }
  pub fn detach_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) -> Result<(), Error> {
    source.as_mut().detach_from_channel(&self.ptr)
  }
}
