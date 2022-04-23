use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::effects::delay_line::DelayLine;
use super::super::signals::synth_signal::SynthSignal;
use super::sound_source::SoundSource;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::time::TimeDelta;

/// Note that `DelayLineTap` is a `SoundSource`, not a `SoundEffect`. A delay line tap can be added to any
/// channel, not only the channel the delay line is on.
pub struct DelayLineTap {
  source: ManuallyDrop<SoundSource>,
  ptr: NonNull<CDelayLineTap>,
  delay_modulator: Option<SynthSignal>,
}
impl DelayLineTap {
  /// Returns a new tap on the DelayLine, at the given position.
  ///
  /// `delay` must be less than or equal to the length of the `DelayLine`.
  pub fn new(delay_line: &mut DelayLine, delay: TimeDelta) -> Option<Self> {
    let ptr = unsafe { Self::fns().addTap.unwrap()(delay_line.cptr(), delay.to_sample_frames()) };
    if ptr.is_null() {
      None
    } else {
      Some(DelayLineTap {
        source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
        ptr: NonNull::new(ptr).unwrap(),
        delay_modulator: None,
      })
    }
  }

  /// Sets the position of the tap on the `DelayLine`, up to the `DelayLine`’s length.
  pub fn set_delay(&mut self, delay: TimeDelta) {
    unsafe { Self::fns().setTapDelay.unwrap()(self.cptr(), delay.to_sample_frames()) }
  }
  /// Sets a signal to modulate the tap delay.
  ///
  /// If the signal is continuous (e.g. an `Envelope` or a triangle `Lfo`, but not a square `Lfo`)
  /// playback is sped up or slowed down to compress or expand time.
  pub fn set_delay_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal| signal.as_ref().cptr());
    unsafe { Self::fns().setTapDelayModulator.unwrap()(self.cptr(), modulator_ptr) }
    self.delay_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the filter delay.
  pub fn delay_modulator(&mut self) -> Option<&SynthSignal> {
    self.delay_modulator.as_ref()
  }

  /// If the `DelayLine` is stereo and flip is set, the tap outputs the `DelayLine`’s left channel
  /// to its right output and vice versa.
  pub fn set_channels_flipped(&mut self, flipped: bool) {
    unsafe { Self::fns().setTapChannelsFlipped.unwrap()(self.cptr(), flipped as i32) }
  }

  pub(crate) fn cptr(&self) -> *mut CDelayLineTap {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_effect_delayline {
    unsafe { &*(*CApiState::get().csound.effect).delayline }
  }
}

impl Drop for DelayLineTap {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freeTap.unwrap()(self.cptr()) }
  }
}

impl AsRef<SoundSource> for DelayLineTap {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for DelayLineTap {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}
