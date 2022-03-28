use crate::capi_state::CApiState;
use crate::ctypes::*;

#[derive(Debug)]
pub struct Sound {
  state: &'static CApiState,
}
impl Sound {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Sound { state }
  }

  /// Returns the sound engine’s current time value, in units of sample frames, 44,100 per second.
  pub fn current_sound_time(&self) -> SampleFrames {
    SampleFrames(unsafe { self.state.csound.getCurrentTime.unwrap()() })
  }

  pub fn default_channel(&self) -> DefaultSoundChannel {
    let ptr = unsafe { self.state.csound.getDefaultChannel.unwrap()() };
    DefaultSoundChannel::from_ptr(ptr, self.state)
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
pub struct DefaultSoundChannel {
  sc: SoundChannelRef,
}
impl DefaultSoundChannel {
  fn from_ptr(ptr: *mut CSoundChannel, state: &'static CApiState) -> Self {
    DefaultSoundChannel {
      sc: SoundChannelRef::from_ptr(ptr, state),
    }
  }
}
impl core::ops::Deref for DefaultSoundChannel {
  type Target = SoundChannelRef;

  fn deref(&self) -> &Self::Target {
    &self.sc
  }
}
impl core::borrow::Borrow<SoundChannelRef> for DefaultSoundChannel {
  fn borrow(&self) -> &SoundChannelRef {
    self // Calls Deref.
  }
}
impl AsRef<SoundChannelRef> for DefaultSoundChannel {
  fn as_ref(&self) -> &SoundChannelRef {
    self //  Call Deref.
  }
}

#[derive(Debug)]
pub struct SoundChannelRef {
  state: &'static CApiState,
  ptr: *mut CSoundChannel,
}
impl SoundChannelRef {
  fn from_ptr(ptr: *mut CSoundChannel, state: &'static CApiState) -> Self {
    SoundChannelRef { state, ptr }
  }
}
