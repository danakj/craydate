use alloc::boxed::Box;
use core::ffi::c_void;
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

  pub fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
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

  /// Creates a new Synth that plays from a SynthGenerator.
  ///
  /// NOTE: THIS CRASHES!! See
  /// https://devforum.play.date/t/c-api-playdate-sound-synth-setgenerator-has-incorrect-api/4482 as
  /// this is believed to be due to some Playdate bug.
  ///
  /// The SynthGenerator is a set of functions that are called in order to fill the sample buffers
  /// with data and react to events on the Synth object.
  pub fn from_generator(generator: SynthGenerator) -> Synth<'sample, 'data> {
    let synth = Self::new();
    unsafe {
      Self::fns().setGenerator.unwrap()(
        synth.ptr,
        // The Playdate API has incorrect types so we need to do some wild casting here:
        // https://devforum.play.date/t/c-api-playdate-sound-synth-setgenerator-has-incorrect-api/4482
        // But also we crash no matter what we pass here, including
        // `Box::into_raw(Box::new(Some(c_render_func)))`.
        c_render_func as *mut Option<CRenderFunc>,
        c_note_on_func as *mut Option<CNoteOnFunc>,
        c_release_func as *mut Option<CReleaseFunc>,
        c_set_parameter_func as *mut Option<CSetParameterFunc>,
        c_dealloc_func as *mut Option<CDeallocFunc>,
        Box::into_raw(Box::new(generator)) as *mut c_void,
      )
    };
    synth
  }

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
  pub fn set_parameter(&mut self, i: u8, value: f32) -> Result<(), Error> {
    let r = unsafe { Self::fns().setParameter.unwrap()(self.ptr, i as i32, value) };
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
    volume: f32, // TODO: Replace this with a type that clamps within 0-1.
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) {
    unsafe {
      Self::fns().playNote.unwrap()(
        self.ptr,
        frequency,
        volume,
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
    note: f32,   // TODO: Make a MidiNote type with note names?
    volume: f32, // TODO: Replace this with a type that clamps within 0-1.
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) {
    unsafe {
      Self::fns().playMIDINote.unwrap()(
        self.ptr,
        note,
        volume,
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
    unsafe { Self::fns().noteOff.unwrap()(self.ptr, when.map_or(0, |w| w.to_sample_frames())) }
  }

  fn fns() -> &'static playdate_sys::playdate_sound_synth {
    unsafe { &*CApiState::get().csound.synth }
  }
}

impl Drop for Synth<'_, '_> {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    // TODO: Does the generator userdata get dropped via `dealloc`?
    unsafe { Self::fns().freeSynth.unwrap()(self.ptr) };
  }
}

impl AsRef<SoundSource> for Synth<'_, '_> {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for Synth<'_, '_> {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}

/// Parameters for the SynthGeneraterRenderFunc.
#[allow(dead_code)]
pub struct SynthRender<'a> {
  /// The left sample buffer in Q8.24 format.
  left: &'a mut [i32],
  /// The right sample buffer in Q8.24 format.
  right: &'a mut [i32],
  /// TODO: What is this?
  rate: u32,
  /// TODO: What is this?
  drate: i32,
  /// The left level value in Q4.28 format, used to scale the samples to follow the synth’s envelope
  /// and/or amplitude modulator levels.
  l: i32,
  /// The left slope value that should be added to `l` every frame.
  dl: i32,
  /// The right level value in Q4.28 format, used to scale the samples to follow the synth’s
  /// envelope and/or amplitude modulator levels.
  r: i32,
  /// The right slope value that should be added to `r` every frame.
  dr: i32,
}

pub struct SynthGeneratorVtable {
  pub render_func: fn(userdata: *const (), SynthRender<'_>) -> i32,
  pub note_on_func: fn(userdata: *const (), note: f32, velocity: f32, length: Option<TimeTicks>),
  pub release_func: fn(userdata: *const (), ended: bool),
  pub set_parameter_func: fn(userdata: *const (), parameter: u8, value: f32) -> bool,
  pub dealloc_func: fn(userdata: *const ()),
}

pub struct SynthGenerator {
  data: *const (),
  vtable: &'static SynthGeneratorVtable,
}
impl SynthGenerator {
  /// Construct a `SynthGenerator` that generates the sample data for a `Synth`.
  ///
  /// The `vtable` defines the behaviour of the generator, and the `data` is a pointer that will
  /// passed to each function in the `vtable`. The `data` pointer is deallocated by the
  /// `SynthGeneratorVtable::dealloc` function.
  pub const unsafe fn new(data: *const (), vtable: &'static SynthGeneratorVtable) -> Self {
    SynthGenerator { data, vtable }
  }
}
impl Drop for SynthGenerator {
  fn drop(&mut self) {
    // The `c_dealloc_func()` will call into there to drop `data` as well.
    (self.vtable.dealloc_func)(self.data)
  }
}

type CRenderFunc =
  unsafe extern "C" fn(*mut c_void, *mut i32, *mut i32, i32, u32, i32, i32, i32, i32, i32) -> i32;
unsafe extern "C" fn c_render_func(
  generator: *mut c_void,
  left: *mut i32,
  right: *mut i32,
  nsamples: i32,
  rate: u32,
  drate: i32,
  l: i32,
  dl: i32,
  r: i32,
  dr: i32,
) -> i32 {
  let generator = generator as *const SynthGenerator;
  let func = (*generator).vtable.render_func;
  let userdata = (*generator).data;
  func(
    userdata,
    SynthRender {
      left: alloc::slice::from_raw_parts_mut(left, nsamples as usize),
      right: alloc::slice::from_raw_parts_mut(right, nsamples as usize),
      rate,
      drate,
      l,
      dl,
      r,
      dr,
    },
  )
}
type CNoteOnFunc = unsafe extern "C" fn(*mut c_void, f32, f32, f32);
unsafe extern "C" fn c_note_on_func(generator: *mut c_void, note: f32, velocity: f32, length: f32) {
  let generator = generator as *const SynthGenerator;
  let func = (*generator).vtable.note_on_func;
  let userdata = (*generator).data;
  // The length is -1 if indefinite, per
  // https://sdk.play.date/1.9.3/Inside%20Playdate%20with%20C.html#f-sound.synth.setGenerator.
  let length = if length == -1.0 {
    None
  } else {
    Some(TimeTicks::from_seconds_lossy(length))
  };
  func(userdata, note, velocity, length)
}
type CReleaseFunc = unsafe extern "C" fn(*mut c_void, i32);
unsafe extern "C" fn c_release_func(generator: *mut c_void, ended: i32) {
  let generator = generator as *const SynthGenerator;
  let func = (*generator).vtable.release_func;
  let userdata = (*generator).data;
  func(userdata, ended != 0)
}
type CSetParameterFunc = unsafe extern "C" fn(*mut c_void, u8, f32) -> i32;
unsafe extern "C" fn c_set_parameter_func(
  generator: *mut c_void,
  parameter: u8,
  value: f32,
) -> i32 {
  let generator = generator as *const SynthGenerator;
  let func = (*generator).vtable.set_parameter_func;
  let userdata = (*generator).data;
  func(userdata, parameter, value) as i32
}
type CDeallocFunc = unsafe extern "C" fn(*mut c_void);
unsafe extern "C" fn c_dealloc_func(generator: *mut c_void) {
  // ```
  // let generator = generator as *mut SynthGenerator;
  // let func = (*generator).vtable.dealloc_func;
  // let userdata = (*generator).data;
  // func(userdata);
  // ```
  // The generator `data` is dealloced by `dealloc_func` in the `Drop::drop` method for
  // SynthGenerator.
  drop(Box::from_raw(generator as *mut SynthGenerator))
}
