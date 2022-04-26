#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::sound_channel::SoundChannel;
use super::sound_source::SoundSource;
use crate::ctypes::*;
use crate::system::System;

/// A `SoundSource` that is a user-defined function that writes to the audio buffer directly.
///
/// Destroying the `CallbackSource` will remove it from the channel if it's attached.
pub struct CallbackSource {
  source: ManuallyDrop<SoundSource>,
  ptr: NonNull<CSoundSource>,
  _stereo_data: Option<Box<StereoData>>,
  _mono_data: Option<Box<MonoData>>,
}
impl CallbackSource {
  /// Constructs a new stereo `CallbackSource` that runs `callback` each sound frame to fill the
  /// stereo sound buffers.
  ///
  /// The `CallbackSource` starts out being attached to the `channel`.
  ///
  /// The `callback` closure should fill the passed-in left and right slices with samples and return
  /// true, or return false if the source is silent through the cycle.
  pub fn new_stereo_for_channel<F>(channel: &mut SoundChannel, callback: F) -> Self
  where
    F: FnMut(&mut [i16], &mut [i16]) -> bool + Sync + 'static,
  {
    let stereo_ptr = Box::into_raw(Box::new(StereoData {
      callback: Box::new(callback),
    }));
    let stereo_data = unsafe { Box::from_raw(stereo_ptr) };
    let ptr = unsafe {
      SoundChannel::fns().addCallbackSource.unwrap()(
        channel.cptr_mut(),
        Some(c_stereo_function),
        stereo_ptr as *mut c_void,
        /*stereo=*/ true as i32,
      )
    };
    let mut s = CallbackSource {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr)),
      ptr: NonNull::new(ptr).unwrap(),
      _stereo_data: Some(stereo_data),
      _mono_data: None,
    };
    // A CallbackSource is already attached when created, but we add it anyway so that the
    // `SoundSource` knows which channel it is attached to. This prevents it from being attached
    // elsewhere and ensures it will be detached on destruction.
    channel.add_source(&mut s).unwrap();
    s
  }

  /// Constructs a new mono `CallbackSource` that runs `callback` each sound frame to fill the mono
  /// sound buffer.
  ///
  /// The `CallbackSource` starts out being attached to the `channel`.
  ///
  /// The `callback` closure should fill the passed-in slice with samples and return true, or return
  /// false if the source is silent through the cycle.
  pub fn new_mono_for_channel<F>(channel: &mut SoundChannel, callback: F) -> Self
  where
    F: FnMut(&mut [i16]) -> bool + Sync + 'static,
  {
    let mono_ptr = Box::into_raw(Box::new(MonoData {
      callback: Box::new(callback),
    }));
    let mono_data = unsafe { Box::from_raw(mono_ptr) };
    let ptr = unsafe {
      SoundChannel::fns().addCallbackSource.unwrap()(
        channel.cptr_mut(),
        Some(c_mono_function),
        mono_ptr as *mut c_void,
        /*stereo=*/ false as i32,
      )
    };
    let mut s = CallbackSource {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr)),
      ptr: NonNull::new(ptr).unwrap(),
      _stereo_data: None,
      _mono_data: Some(mono_data),
    };
    // A CallbackSource is already attached when created, but we add it anyway so that the
    // `SoundSource` knows which channel it is attached to. This prevents it from being attached
    // elsewhere and ensures it will be detached on destruction.
    channel.add_source(&mut s).unwrap();
    s
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut CSoundSource {
    self.ptr.as_ptr()
  }
}

impl Drop for CallbackSource {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { System::fns().realloc.unwrap()(self.cptr_mut() as *mut c_void, 0) };
  }
}

impl AsRef<SoundSource> for CallbackSource {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for CallbackSource {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}

struct StereoData {
  callback: Box<dyn FnMut(&mut [i16], &mut [i16]) -> bool + Sync>,
}

unsafe extern "C" fn c_stereo_function(
  c_data: *mut c_void,
  left: *mut i16,
  right: *mut i16,
  len: i32,
) -> i32 {
  let left = unsafe { core::slice::from_raw_parts_mut(left, len as usize) };
  let right = unsafe { core::slice::from_raw_parts_mut(right, len as usize) };
  let c_data = c_data as *mut StereoData;
  unsafe { ((*c_data).callback)(left, right) as i32 }
}

struct MonoData {
  callback: Box<dyn FnMut(&mut [i16]) -> bool + Sync>,
}

unsafe extern "C" fn c_mono_function(
  c_data: *mut c_void,
  left: *mut i16,
  _right: *mut i16,
  len: i32,
) -> i32 {
  let left = unsafe { core::slice::from_raw_parts_mut(left, len as usize) };
  let c_data = c_data as *mut MonoData;
  unsafe { ((*c_data).callback)(left) as i32 }
}
