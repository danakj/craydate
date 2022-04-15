use core::ptr::NonNull;

use alloc::rc::{Rc, Weak};

use super::super::{SoundCompletionCallback, StereoVolume};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;

/// Represents a weak connection to whatever is playing the SoundSource.
///
/// Note that we avoid holding a borrow on the player, or ownership of its Rc, so that it can be
/// destroyed while playing a SoundSource.
#[derive(Debug)]
enum Attachment {
  None,
  Channel(Weak<NonNull<CSoundChannel>>),
  Instrument,
}
impl Attachment {
  fn is_none(&self) -> bool {
    match self {
      Self::None => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
pub struct SoundSource {
  ptr: *mut CSoundSource,
  // The `channel` is set when the SoundSource has been added to the SoundChannel.
  attachment: Attachment,
  // When the RegisteredCallback is destroyed, the user-given closure will be destroyed as well.
  completion_callback: Option<RegisteredCallback>,
}
impl SoundSource {
  pub(crate) fn from_ptr(ptr: *mut CSoundSource) -> Self {
    SoundSource {
      ptr,
      attachment: Attachment::None,
      completion_callback: None,
    }
  }
  pub(crate) fn cptr(&self) -> *mut CSoundSource {
    self.ptr
  }

  /// Attach the SoundSource to the `channel` if it is not already attached to a `SoundChannel` or
  /// `Instrument`.
  pub(crate) fn attach_to_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    // Mimic the Playdate API behaviour. Attaching a Source to a Channel when it's already attached
    // does nothing.
    match self.attachment {
      Attachment::None => {
        // The SoundSource holds a Weak pointer to the SoundChannel so it knows whether to remove
        // itself in drop().
        self.attachment = Attachment::Channel(Rc::downgrade(channel));
        let r =
          unsafe { (*CApiState::get().csound.channel).addSource.unwrap()(channel.as_ptr(), self.cptr()) };
        assert!(r != 0);
        Ok(())
      }
      _ => Err(Error::AlreadyAttachedError),
    }
  }
  /// Removes the SoundSource from the `channel` if it was currently attached.
  ///
  /// If the SoundSource is not attached to `channel`, then `Error::NotFoundError` is returned.
  pub(crate) fn detach_from_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    match &mut self.attachment {
      Attachment::Channel(weak_ptr) if weak_ptr.ptr_eq(&Rc::downgrade(&channel)) => {
        let r = unsafe {
          (*CApiState::get().csound.channel).removeSource.unwrap()(channel.as_ptr(), self.cptr())
        };
        self.attachment = Attachment::None;
        assert!(r != 0);
        return Ok(());
      }
      _ => Err(Error::NotFoundError),
    }
  }

  /// Attach the SoundSource to the `instrument` if it is not already attached to a `SoundChannel`
  /// or `Instrument`.
  pub(crate) fn attach_to_instrument(&mut self) -> bool {
    // Mimic the Playdate API behaviour. Attaching a Source to a Channel when it's already attached
    // does nothing.
    if self.attachment.is_none() {
      self.attachment = Attachment::Instrument;
      true
    } else {
      false
    }
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

    match &self.attachment {
      Attachment::None => (),
      Attachment::Channel(weak_ptr) => {
        if let Some(rc_ptr) = weak_ptr.upgrade() {
          let r = self.detach_from_channel(&rc_ptr);
          assert!(r.is_ok()); // Otherwise, `self.channel` was lying.
        }
      }
      Attachment::Instrument => {
        // Synth claims that it removes itself from the sound system, and there's no function to
        // remove it from the Instrument ourselves:
        // https://sdk.play.date/1.9.3/Inside%20Playdate%20with%20C.html#f-sound.synth.freeSynth

        // TODO: It's wrong, Playdate plays garbage if you drop the Synths that were added to
        // instruments.
      }
    }
  }
}
