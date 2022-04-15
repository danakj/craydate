use core::mem::MaybeUninit;

use alloc::vec::Vec;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;
use crate::time::TimeTicks;

pub struct AudioSample {
  ptr: *mut CAudioSample,
  data: Vec<u8>,
}
impl AudioSample {
    pub(crate) fn from_ptr(ptr: *mut CAudioSample) -> AudioSample {
    AudioSample {
      ptr,
      data: Vec::new(),
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
  pub fn from_file(path: &str) -> Option<AudioSample> {
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
  pub fn from_vec(data: Vec<u8>, format: SoundFormat, sample_rate: u32) -> AudioSample {
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
    let mut sample = Self::from_ptr(ptr);
    sample.data = data;
    sample
  }

  /// Creates a new AudioSample referencing the given audio data.
  ///
  /// The AudioSample keeps a pointer to the data instead of copying it.
  pub fn from_slice(data: &[u8], format: SoundFormat, sample_rate: u32) -> AudioSample {
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
    let mut sample = Self::from_ptr(ptr);
    sample.data.reserve(data.len());
    sample.data.extend(data.iter());
    sample
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
  // Note: No mutable access to the buffer is provided because audio runs on a different thread, so
  // changing data in the AudioSample is probably not intended and would be a data race.
  pub fn data(&self) -> &[u8] {
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
impl Drop for AudioSample {
  fn drop(&mut self) {
    // Note: The sample is destroyed before the data we own that it refers to.
    unsafe { (*CApiState::get().csound.sample).freeSample.unwrap()(self.ptr) }
  }
}
