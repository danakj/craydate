use alloc::rc::{Rc, Weak};
use core::ptr::NonNull;

use super::super::{SoundCompletionCallback, StereoVolume};
use crate::callback_builder::Constructed;
use crate::callbacks::RegisteredCallback;
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
}
impl Attachment {
  fn is_none(&self) -> bool {
    match self {
      Self::None => true,
      _ => false,
    }
  }
}

/// A `SoundSource` produces sound that can be played into a `SoundChannel`, thus playing to the
/// device's sound outputs.
///
/// There are many types which act as a `SoundSource`. Any such type would implement
/// `AsRef<SoundSource>` and `AsMut<SoundSource>`. They also have `as_source()` and
/// `as_source_mut()` methods, through the `AsSoundSource` trait, to access the `SoundSource`
/// methods more easily.
#[derive(Debug)]
pub struct SoundSource {
  ptr: NonNull<CSoundSource>,
  // The `channel` is set when the SoundSource has been added to the SoundChannel.
  attachment: Attachment,
  // When the RegisteredCallback is destroyed, the user-given closure will be destroyed as well.
  completion_callback: Option<RegisteredCallback>,
}
impl SoundSource {
  pub(crate) fn from_ptr(ptr: *mut CSoundSource) -> Self {
    SoundSource {
      ptr: NonNull::new(ptr).unwrap(),
      attachment: Attachment::None,
      completion_callback: None,
    }
  }

  /// Attach the SoundSource to the `channel` if it is not already attached to a `SoundChannel` or
  /// `Instrument`.
  pub(crate) fn attach_to_channel(
    &mut self,
    channel: &Rc<NonNull<CSoundChannel>>,
  ) -> Result<(), Error> {
    // Mimic the Playdate C Api behaviour. Attaching a Source to a Channel when it's already
    // attached does nothing.
    match self.attachment {
      Attachment::None => {
        // The SoundSource holds a Weak pointer to the SoundChannel so it knows whether to remove
        // itself in drop().
        self.attachment = Attachment::Channel(Rc::downgrade(channel));
        let r = unsafe {
          (*CApiState::get().csound.channel).addSource.unwrap()(channel.as_ptr(), self.cptr_mut())
        };
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
          (*CApiState::get().csound.channel).removeSource.unwrap()(
            channel.as_ptr(),
            self.cptr_mut(),
          )
        };
        self.attachment = Attachment::None;
        assert!(r != 0);
        return Ok(());
      }
      _ => Err(Error::NotFoundError),
    }
  }

  /// Return if the SoundSouce is currently attached to a `SoundChannel`.
  pub(crate) fn is_attached(&self) -> bool {
    !self.attachment.is_none()
  }

  /// Gets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn volume(&self) -> StereoVolume {
    let mut v = StereoVolume::zero();
    unsafe {
      // getVolume() takes a mutable pointer it changes no visible state.
      Self::fns().getVolume.unwrap()(
        self.cptr() as *mut _,
        v.left.as_mut_ptr(),
        v.right.as_mut_ptr(),
      )
    };
    v
  }
  /// Sets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn set_volume(&mut self, v: StereoVolume) {
    unsafe { Self::fns().setVolume.unwrap()(self.cptr_mut(), v.left.into(), v.right.into()) }
  }
  /// Returns whether the source is currently playing.
  pub fn is_playing(&self) -> bool {
    // isPlaying() takes a mutable pointer it changes no visible state.
    unsafe { Self::fns().isPlaying.unwrap()(self.cptr() as *mut _) != 0 }
  }

  /// Sets a callback to be called when the `SoundSource` finishes playing.
  ///
  /// The callback will be registered as a system event, and the application will be notified to run
  /// the callback via a `SystemEvent::Callback` event. When that occurs, the application's
  /// `Callbacks` object which was used to construct the `completion_callback` can be `run()` to
  /// execute the closure bound in the `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// source.set_completion_callback(SoundCompletionCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("finished");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.run(12);
  ///   }
  /// }
  /// ```
  pub fn set_completion_callback<'a, T, F: Fn(T) + 'static>(
    &mut self,
    completion_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    self.completion_callback = None;
    let func = completion_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.cptr_mut() as usize;
      let (func, reg) = callbacks.add_sound_source_completion(key, cb);
      self.completion_callback = Some(reg);
      Some(func)
    });
    unsafe { Self::fns().setFinishCallback.unwrap()(self.cptr_mut(), func) }
  }

  pub(crate) fn cptr(&self) -> *const CSoundSource {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CSoundSource {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_sound_source {
    unsafe { &*CApiState::get().csound.source }
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
    }
  }
}

/// Provides explicit access to a type's `SoundSource` methods when it can act as a `SoundSource`.
pub trait AsSoundSource: AsRef<SoundSource> + AsMut<SoundSource> {
  fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
  }
}
impl<T> AsSoundSource for T where T: AsRef<SoundSource> + AsMut<SoundSource> {}
