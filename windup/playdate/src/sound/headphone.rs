use alloc::boxed::Box;
use core::ffi::c_void;

use super::Sound;
use crate::capi_state::CApiState;

type ActiveMicrophoneInnerBox = Box<dyn Fn(&[i16]) -> MicrophoneCallbackOutput + Sync>;

pub enum MicrophoneCallbackOutput {
  StopRecording,
  ContinueRecording,
}

pub struct ActiveMicrophoneCallback {
  generation: usize,
  // Holds the data alive while the callback is set. The pointer in this box is passed to the C
  // function by Playdate.
  _c_function_data: Box<ActiveMicrophoneInnerBox>,
}
impl ActiveMicrophoneCallback {
  pub(crate) fn set_active_callback<F: Fn(&[i16]) -> MicrophoneCallbackOutput + Sync + 'static>(
    closure: F,
    force_device_microphone: bool,
  ) -> Self {
    let gen = CApiState::get().headphone_change_generation.get() + 1;
    CApiState::get().headphone_change_generation.set(gen);

    // A wide pointer.
    let inner: ActiveMicrophoneInnerBox = Box::new(closure);
    // Boxed a second time to get a narrow pointer, which we can give to C, and unwrapped.
    let c_function_data: *mut ActiveMicrophoneInnerBox = Box::into_raw(Box::new(inner));
    // Ownership of the `c_function_data`.
    let boxed_c_function_data = unsafe { Box::from_raw(c_function_data) };

    unsafe extern "C" fn c_func(c_data: *mut c_void, buf: *mut i16, len: i32) -> i32 {
      crate::log::log_to_stdout_with_newline("c_func");
      let closure = c_data as *mut ActiveMicrophoneInnerBox;
      let out = (*closure)(core::slice::from_raw_parts(buf, len as usize));
      match out {
        MicrophoneCallbackOutput::ContinueRecording => 1,
        MicrophoneCallbackOutput::StopRecording => 0,
      }
    }
    unsafe {
      Sound::fns().setMicCallback.unwrap()(
        Some(c_func),
        c_function_data as *mut c_void,
        force_device_microphone as i32,
      )
    };

    ActiveMicrophoneCallback {
      generation: gen,
      _c_function_data: boxed_c_function_data,
    }
  }
}

impl Drop for ActiveMicrophoneCallback {
  fn drop(&mut self) {
    // Use a generation tag to avoid unsetting the headphone callback if another callback was set
    // before this object was dropped.
    if self.generation == CApiState::get().headphone_change_generation.get() {
      unsafe { Sound::fns().setMicCallback.unwrap()(None, core::ptr::null_mut(), false as i32) }
    }
  }
}
