use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::time::TimeTicks;

pub struct AudioSample<'data> {
  ptr: *mut CAudioSample,
  _marker: PhantomData<&'data u8>,
}
impl<'data> AudioSample<'data> {
    pub(crate) fn from_ptr<'a>(ptr: *mut CAudioSample) -> AudioSample<'a> {
    AudioSample {
      ptr,
      _marker: PhantomData,
    }
  }
  pub(crate) fn cptr(&self) -> *mut CAudioSample {
      self.ptr
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
