use core::marker::PhantomData;
use core::mem::ManuallyDrop;

use super::super::audio_sample::AudioSample;
use super::super::sound_range::SoundRange;
use super::sound_source::SoundSource;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::ctypes_enums::SoundWaveform;
use crate::error::Error;
use crate::time::{TimeDelta, TimeTicks};

#[derive(Debug)]
pub struct Synth<'sample, 'data> {
  source: ManuallyDrop<SoundSource>,
  ptr: *mut CSynth,
  _marker: PhantomData<&'sample AudioSample<'data>>,
}
impl<'sample, 'data> Synth<'sample, 'data> {
  /// Creates a new Synth.
  fn new() -> Synth<'sample, 'data> {
    let ptr = unsafe { Self::fns().newSynth.unwrap()() };
    Synth {
      source: ManuallyDrop::new(SoundSource::new(ptr as *mut CSoundSource)),
      ptr,
      _marker: PhantomData,
    }
  }

  /// Creates a new Synth that plays a waveform.
  pub fn from_waveform(waveform: SoundWaveform) -> Synth<'sample, 'data> {
    let synth = Self::new();
    unsafe { Self::fns().setWaveform.unwrap()(synth.ptr, waveform) };
    synth
  }

  /// Creates a new Synth that plays a sample.
  ///
  /// An optional sustain region defines a loop to play while the note is on. Sample data must be
  /// uncompressed PCM, not ADPCM.
  pub fn from_sample(
    sample: &'sample AudioSample<'data>,
    sustain_region: Option<SoundRange>,
  ) -> Synth<'sample, 'data> {
    let synth = Self::new();
    unsafe {
      Self::fns().setSample.unwrap()(
        synth.ptr,
        sample.cptr(),
        sustain_region.as_ref().map_or(0, |r| r.start.to_sample_frames()),
        sustain_region.as_ref().map_or(0, |r| r.end.to_sample_frames()),
      )
    };
    synth
  }

  // TODO: set_generator() as from_generator()

  /// Sets the attack time for the sound envelope.
  pub fn set_attack_time(&mut self, attack_time: TimeDelta) {
    unsafe { Self::fns().setAttackTime.unwrap()(self.ptr, attack_time.to_seconds()) }
  }
  /// Sets the decay time for the sound envelope.
  pub fn set_decay_time(&mut self, decay_time: TimeDelta) {
    unsafe { Self::fns().setDecayTime.unwrap()(self.ptr, decay_time.to_seconds()) }
  }
  /// Sets the sustain level, from 0 to 1, for the sound envelope.
  pub fn set_sustain_level(&mut self, level: f32) {
    unsafe { Self::fns().setSustainLevel.unwrap()(self.ptr, level) }
  }
  /// Sets the release time for the sound envelope.
  pub fn set_release_time(&mut self, release_time: TimeDelta) {
    unsafe { Self::fns().setReleaseTime.unwrap()(self.ptr, release_time.to_seconds()) }
  }
  /// Transposes the synth’s output by the given number of half steps.
  ///
  /// For example, if the transpose is set to 2 and a C note is played, the synth will output a D
  /// instead.
  pub fn set_transpose(&mut self, half_steps: f32) {
    unsafe { Self::fns().setTranspose.unwrap()(self.ptr, half_steps) }
  }

  // TODO: setFrequencyModulator
  // TODO: getFrequencyModulator

  // TODO: setAmplitudeModulator
  // TODO: getAmplitudeModulator

  // TODO: setParameterModulator
  // TODO: getParameterModulator

  /// Returns the number of parameters advertised by the Synth.
  pub fn num_parameters(&self) -> i32 {
    unsafe { Self::fns().getParameterCount.unwrap()(self.ptr) }
  }
  /// Set the Synth's `i`th parameter to `value`.
  ///
  /// `i` is 0-based, so the first parameter is `0`, the second is `1`, etc. Returns
  /// `Error::NotFoundError` is the parameter `i` is not valid.
  pub fn set_parameter(&mut self, i: i32, value: f32) -> Result<(), Error> {
    let r = unsafe { Self::fns().setParameter.unwrap()(self.ptr, i, value) };
    match r {
      0 => Err(Error::NotFoundError),
      _ => Ok(()),
    }
  }

  /// Plays a note on the Synth, using the `frequency`.
  ///
  /// If `length` is `None`, the note will continue playing until a subsequent `stop()` call. If
  /// `when` is None, the note is played immediately, otherwise the note is scheduled for the given
  /// absolute time. Use `Sound::current_sound_time()` to get the current time.
  pub fn play_frequency_note(
    &mut self,
    frequency: f32,
    vel: f32,
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) {
    unsafe {
      Self::fns().playNote.unwrap()(
        self.ptr,
        frequency,
        vel,
        length.map_or(-1.0, |l| l.to_seconds()),
        when.map_or(0, |w| w.to_sample_frames()),
      )
    }
  }

  /// Plays a MIDI note on the Synth, where for `note`: 'C4' is `60.0`.
  ///
  /// If `length` is `None`, the note will continue playing until a subsequent `stop()` call. If
  /// `when` is None, the note is played immediately, otherwise the note is scheduled for the given
  /// absolute time. Use `Sound::current_sound_time()` to get the current time.
  pub fn play_midi_note(
    &mut self,
    note: f32, // TODO: Make a MidiNote type with note names?
    vel: f32,
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) {
    unsafe {
      Self::fns().playMIDINote.unwrap()(
        self.ptr,
        note,
        vel,
        length.map_or(-1.0, |l| l.to_seconds()),
        when.map_or(0, |w| w.to_sample_frames()),
      )
    }
  }

  /// Stops the currently play8iung note.
  /// 
  /// If `when` is `None`, the note is stopped immediately. Otherwise it is scheduled to be stopped
  /// at the given absolute time. Use `Sound::current_sound_time()` to get the current time.
  pub fn stop(&mut self, when: Option<TimeTicks>) {
    unsafe {
      Self::fns().noteOff.unwrap()(
        self.ptr,
        when.map_or(0, |w| w.to_sample_frames()),
      )
    }
  }

  fn fns() -> &'static playdate_sys::playdate_sound_synth {
    unsafe { &*CApiState::get().csound.synth }
  }
}

impl Drop for Synth<'_, '_> {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freeSynth.unwrap()(self.ptr) };
  }
}
