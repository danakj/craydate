use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use super::super::midi::midi_note_range::MidiNoteRange;
use super::super::StereoVolume;
use super::sound_source::SoundSource;
use super::synth::Synth;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::time::{TimeDelta, TimeTicks};

#[derive(Debug)]
pub struct InstrumentRef {
  ptr: NonNull<CSynthInstrument>,
}
impl InstrumentRef {
  pub(crate) fn from_ptr(ptr: *mut CSynthInstrument) -> Self {
    InstrumentRef {
      ptr: NonNull::new(ptr).unwrap(),
    }
  }

  /// Adds the given `Synth` to the instrument.
  ///
  /// The synth will respond to play events in the given `midi_range`, inclusive. The `transpose`
  /// argument is in half-step units, and is added to the instrument’s transpose parameter.
  ///
  /// # Return
  /// On success, returns an id that will be used to refer to the attached Synth. The function
  /// returns `Error::AlreadyAttachedError` if the `Synth` is already attached to another
  /// `Instrument` or `SoundChannel`.
  pub fn add_voice(
    &mut self,
    synth: &mut Synth,
    midi_range: MidiNoteRange,
    transpose: f32,
  ) -> Result<usize, Error> {
    if synth.as_mut().attach_to_instrument() {
      let (start, end) = midi_range.to_start_end();
      let r = unsafe {
        Instrument::fns().addVoice.unwrap()(self.cptr(), synth.cptr(), start, end, transpose)
      };
      assert!(r != 0);
      Ok(synth as *mut Synth as usize)
    } else {
      Err(Error::AlreadyAttachedError)
    }
  }

  /// Plays a note on the Instrument, using the `frequency`.
  ///
  /// The instrument passes the play event to the `Synth` in its collection that has been off for
  /// the longest, or has been playing longest if all synths are currently playing.
  ///
  /// If `length` is `None`, the note will continue playing until a subsequent `stop()` call. If
  /// `when` is None, the note is played immediately, otherwise the note is scheduled for the given
  /// absolute time. Use `Sound::current_sound_time()` to get the current time.
  ///
  /// An id for the `Synth` that received the play event is returned. The id matches the one
  /// returned from add_voice() for the `Synth`.
  pub fn play_frequency_note(
    &mut self,
    frequency: f32,
    volume: f32, // TODO: Replace this with a type that clamps within 0-1.
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) -> usize {
    let synth_ptr = unsafe {
      Instrument::fns().playNote.unwrap()(
        self.cptr(),
        frequency,
        volume,
        length.map_or(-1.0, |l| l.to_seconds()),
        when.map_or(0, |w| w.to_sample_frames()),
      )
    };
    synth_ptr as usize
  }

  /// Plays a MIDI note on the Instrument, where 'C4' is `60.0` for the `note`.
  ///
  /// The instrument passes the play event to the `Synth` in its collection that has been off for
  /// the longest, or has been playing longest if all synths are currently playing.
  ///
  /// If `length` is `None`, the note will continue playing until a subsequent `stop()` call. If
  /// `when` is None, the note is played immediately, otherwise the note is scheduled for the given
  /// absolute time. Use `Sound::current_sound_time()` to get the current time.
  ///
  /// An id for the `Synth` that received the play event is returned. The id matches the one
  /// returned from add_voice() for the `Synth`.
  pub fn play_midi_note(
    &mut self,
    midi_note: f32,
    volume: f32, // TODO: Replace this with a type that clamps within 0-1.
    length: Option<TimeDelta>,
    when: Option<TimeTicks>,
  ) -> usize {
    let synth_ptr = unsafe {
      Instrument::fns().playMIDINote.unwrap()(
        self.cptr(),
        midi_note,
        volume,
        length.map_or(-1.0, |l| l.to_seconds()),
        when.map_or(0, |w| w.to_sample_frames()),
      )
    };
    synth_ptr as usize
  }

  /// Forwards a stop event to the `Synth` currently playing the given note.
  ///
  /// See also `Synth::stop()`.
  ///
  /// If `when` is `None`, the note is stopped immediately. Otherwise it is scheduled to be stopped
  /// at the given absolute time. Use `Sound::current_sound_time()` to get the current time.
  pub fn stop_note(&mut self, midi_note: f32, when: Option<TimeTicks>) {
    unsafe {
      Instrument::fns().noteOff.unwrap()(
        self.cptr(),
        midi_note,
        when.map_or(0, |w| w.to_sample_frames()),
      )
    }
  }
  /// Sends a stop event to all `Synth` voices in the instrument.
  ///
  /// See also `Synth::stop()`.
  ///
  /// If `when` is `None`, the note is stopped immediately. Otherwise it is scheduled to be stopped
  /// at the given absolute time. Use `Sound::current_sound_time()` to get the current time.
  pub fn stop_all_notes(&mut self, when: Option<TimeTicks>) {
    unsafe {
      Instrument::fns().allNotesOff.unwrap()(self.cptr(), when.map_or(0, |w| w.to_sample_frames()))
    }
  }

  /// Sets the pitch bend to be applied to the voices in the instrument.
  pub fn set_pitch_bend(&mut self, bend: f32) {
    unsafe { Instrument::fns().setPitchBend.unwrap()(self.cptr(), bend) }
  }
  /// Sets the pitch bend range to be applied to the voices in the instrument.
  pub fn set_pitch_bend_range(&mut self, half_steps: f32) {
    unsafe { Instrument::fns().setPitchBendRange.unwrap()(self.cptr(), half_steps) }
  }
  /// Sets the transpose parameter for all voices in the instrument.
  pub fn set_transpose(&mut self, half_steps: f32) {
    unsafe { Instrument::fns().setTranspose.unwrap()(self.cptr(), half_steps) }
  }

  /// Returns the number of voices in the instrument currently playing.
  pub fn active_voice_count(&self) -> i32 {
    unsafe { Instrument::fns().activeVoiceCount.unwrap()(self.cptr()) }
  }

  /// Gets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn volume(&self) -> StereoVolume {
    let mut v = StereoVolume {
      left: 0.0,
      right: 0.0,
    };
    unsafe { Instrument::fns().getVolume.unwrap()(self.cptr(), &mut v.left, &mut v.right) };
    v
  }
  /// Sets the playback volume (0.0 - 1.0) for left and right channels of the source.
  pub fn set_volume(&mut self, v: StereoVolume) {
    unsafe {
      Instrument::fns().setVolume.unwrap()(
        self.cptr(),
        v.left.clamp(0f32, 1f32),
        v.right.clamp(0f32, 1f32),
      )
    }
  }

  pub(crate) fn cptr(&self) -> *mut CSynthInstrument {
    self.ptr.as_ptr()
  }
}

/// `Instrument` collects a number of `Synth` objects together to provide polyphony. It can be
/// attached to one or more `SequenceTrack`s.
#[derive(Debug)]
pub struct Instrument {
  source: ManuallyDrop<SoundSource>,
  iref: InstrumentRef,
}
impl<'data> Instrument {
  pub fn as_source(&self) -> &SoundSource {
    self.as_ref()
  }
  pub fn as_source_mut(&mut self) -> &mut SoundSource {
    self.as_mut()
  }

  /// Creates a new Instrument.
  ///
  /// Destroying an instrument removes it from all SequenceTracks it may have been added to.
  pub fn new() -> Self {
    let ptr = unsafe { Self::fns().newInstrument.unwrap()() };
    Instrument {
      source: ManuallyDrop::new(SoundSource::from_ptr(ptr as *mut CSoundSource)),
      iref: InstrumentRef::from_ptr(ptr),
    }
  }

  fn fns() -> &'static playdate_sys::playdate_sound_instrument {
    unsafe { &*CApiState::get().csound.instrument }
  }
}

impl Drop for Instrument {
  fn drop(&mut self) {
    // Ensure the SoundSource has a chance to clean up before it is freed.
    unsafe { ManuallyDrop::drop(&mut self.source) };
    unsafe { Self::fns().freeInstrument.unwrap()(self.iref.cptr()) }
  }
}

impl AsRef<SoundSource> for Instrument {
  fn as_ref(&self) -> &SoundSource {
    &self.source
  }
}
impl AsMut<SoundSource> for Instrument {
  fn as_mut(&mut self) -> &mut SoundSource {
    &mut self.source
  }
}

impl core::ops::Deref for Instrument {
  type Target = InstrumentRef;

  fn deref(&self) -> &Self::Target {
    &self.iref
  }
}
impl core::ops::DerefMut for Instrument {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.iref
  }
}
impl AsRef<InstrumentRef> for Instrument {
  fn as_ref(&self) -> &InstrumentRef {
    self
  }
}
impl AsMut<InstrumentRef> for Instrument {
  fn as_mut(&mut self) -> &mut InstrumentRef {
    self
  }
}
impl core::borrow::Borrow<InstrumentRef> for Instrument {
  fn borrow(&self) -> &InstrumentRef {
    self
  }
}
impl core::borrow::BorrowMut<InstrumentRef> for Instrument {
  fn borrow_mut(&mut self) -> &mut InstrumentRef {
    self
  }
}
