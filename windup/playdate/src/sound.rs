use alloc::collections::BTreeMap;
use core::cell::{Ref, RefCell, RefMut};

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelId(usize);
impl ChannelId {
  // 0 is never returned by gen_next_channel_id(), and will refer to the default system-owned
  // channel.
  const DEFAULT_CHANNEL: ChannelId = ChannelId(0);
}
impl From<usize> for ChannelId {
  fn from(u: usize) -> Self {
    ChannelId(u)
  }
}

#[derive(Debug)]
pub struct Channels<Key = ChannelId, Value = *mut CSoundChannel> {
  state: &'static CApiState,
  last_id: Key,
  map: BTreeMap<Key, RefCell<Value>>,
}
impl Channels<ChannelId, *mut CSoundChannel> {
  fn new(state: &'static CApiState) -> Self {
    let mut map = BTreeMap::new();
    map.insert(
      ChannelId::DEFAULT_CHANNEL,
      RefCell::new(unsafe { state.csound.getDefaultChannel.unwrap()() }),
    );
    Channels {
      state,
      last_id: ChannelId::DEFAULT_CHANNEL,
      map,
    }
  }
  fn gen_next_channel_id(&mut self) -> ChannelId {
    // This function must never return DEFAULT_CHANNEL.
    self.last_id = ChannelId(self.last_id.0 + 1);
    assert!(self.last_id != ChannelId::DEFAULT_CHANNEL);
    self.last_id
  }

  pub fn default_channel(&self) -> Result<SoundChannel<'_>, Error> {
    self.channel(ChannelId::DEFAULT_CHANNEL)
  }

  pub fn default_channel_mut(&self) -> Result<SoundChannelMut<'_>, Error> {
    self.channel_mut(ChannelId::DEFAULT_CHANNEL)
  }

  pub fn channel(&self, channel_id: ChannelId) -> Result<SoundChannel<'_>, Error> {
    let borrow = self.map.get(&channel_id).unwrap().try_borrow()?;
    let ptr = *borrow;
    Ok(SoundChannel {
      state: self.state,
      ptr,
      _borrow: Some(borrow),
    })
  }

  pub fn channel_mut(&self, channel_id: ChannelId) -> Result<SoundChannelMut<'_>, Error> {
    let borrow_mut = self.map.get(&channel_id).unwrap().try_borrow_mut()?;
    let ptr = *borrow_mut;
    Ok(SoundChannelMut {
      immut: SoundChannel {
        state: self.state,
        ptr,
        _borrow: None,
      },
      _borrow_mut: borrow_mut,
    })
  }

  pub fn new_channel(&mut self) -> (ChannelId, SoundChannelMut<'_>) {
    let id = self.gen_next_channel_id();
    let channel_api = self.state.csound.channel;
    let ptr = unsafe { (*channel_api).newChannel.unwrap()() };
    let borrow_mut = self.map.entry(id).or_insert(RefCell::new(ptr)).borrow_mut();
    let channel = SoundChannelMut {
      immut: SoundChannel {
        state: self.state,
        ptr,
        _borrow: None,
      },
      _borrow_mut: borrow_mut,
    };
    (id, channel)
  }

  pub fn delete_channel(&mut self, channel_id: ChannelId) -> Result<(), Error> {
    let channel_api = self.state.csound.channel;
    let cell = self.map.remove(&channel_id).ok_or(Error::NotFoundError())?;
    // No try_borrow() here, as `&mut self` implies there's no references held to any channels at
    // the moment. Channel references hold a stacked borrow of `self`.
    let ptr = *cell.borrow();
    unsafe { (*channel_api).freeChannel.unwrap()(ptr) };
    Ok(())
  }
}

#[derive(Debug)]
pub struct Sound {
  state: &'static CApiState,
  pub channels: Channels,
}
impl Sound {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Sound {
      state,
      channels: Channels::new(state),
    }
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
pub struct SoundChannel<'a> {
  state: &'static CApiState,
  ptr: *mut CSoundChannel,
  // This field is None when the SoundChannel is the field of a SoundChannelMut. The borrow in that
  // case lives in the SoundChannelMut.
  _borrow: Option<Ref<'a, *mut CSoundChannel>>,
}
impl SoundChannel<'_> {
  /// Gets the volume for the channel, in the range [0-1].
  pub fn volume(&self) -> f32 {
    let channel_api = self.state.csound.channel;
    unsafe { (*channel_api).getVolume.unwrap()(self.ptr) }
  }
}

#[derive(Debug)]
pub struct SoundChannelMut<'a> {
  immut: SoundChannel<'a>,
  _borrow_mut: RefMut<'a, *mut CSoundChannel>,
}
impl<'a> core::ops::Deref for SoundChannelMut<'a> {
  type Target = SoundChannel<'a>;

  fn deref(&self) -> &Self::Target {
    &self.immut
  }
}
