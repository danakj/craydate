use alloc::rc::{Rc, Weak};

use super::super::{SoundCompletionCallback, StereoVolume};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;

#[derive(Debug)]
pub struct SoundSource {
  ptr: *mut CSoundSource,
  // The `channel` is set when the SoundSource has been added to the SoundChannel.
  channel: Option<Weak<*mut CSoundChannel>>, // Don't hold a borrow on SoundChannel.
  // When the RegisteredCallback is destroyed, the user-given closure will be destroyed as well.
  completion_callback: Option<RegisteredCallback>,
}
impl SoundSource {
  pub(crate) fn new(ptr: *mut CSoundSource) -> Self {
    SoundSource {
      ptr,
      channel: None,
      completion_callback: None,
    }
  }
  pub(crate) fn cptr(&self) -> *mut CSoundSource {
    self.ptr
  }

  /// Attach the SoundSource to the `channel` if it is not already attached to a channel.
  pub(crate) fn attach_to_channel(&mut self, channel: Weak<*mut CSoundChannel>) {
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
  pub(crate) fn detach_from_channel(
    &mut self,
    channel: Rc<*mut CSoundChannel>,
  ) -> Result<(), Error> {
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
