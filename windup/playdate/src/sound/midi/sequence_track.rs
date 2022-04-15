use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::{Instrument, InstrumentRef};
use crate::capi_state::CApiState;
use crate::ctypes::*;

#[derive(Debug)]
pub struct TrackNote {
  /// The length of the note in steps, not time - ​that is, it follows the sequence’s tempo.
  length: u32,
  // TODO: Support MIDI string notation (e.g. "Db3").
  midi_note: f32,
  velocity: f32,
}
impl Default for TrackNote {
  fn default() -> Self {
    Self {
      length: 1,
      midi_note: 0.0,
      velocity: 1.0,
    }
  }
}

pub struct SequenceTrackRef {
  ptr: NonNull<CSequenceTrack>,
  instrument_ref: Option<InstrumentRef>,
}
impl SequenceTrackRef {
  pub fn new(ptr: *mut CSequenceTrack, instrument_ref: Option<InstrumentRef>) -> Self {
    SequenceTrackRef {
      ptr: NonNull::new(ptr).unwrap(),
      instrument_ref,
    }
  }
  pub(crate) fn cptr(&self) -> *mut CSequenceTrack {
    self.ptr.as_ptr()
  }

  /// Gets the `Instrument` assigned to the track.
  /// 
  /// New tracks do not have an instrument.
  pub fn instrument(&self) -> Option<&InstrumentRef> {
    self.instrument_ref.as_ref()
  }
  /// Gets the `Instrument` assigned to the track.
  /// 
  /// New tracks do not have an instrument.
  pub fn instrument_mut(&mut self) -> Option<&mut InstrumentRef> {
    self.instrument_ref.as_mut()
  }

  pub fn set_instrument(&mut self, instrument: &mut InstrumentRef) {
    unsafe { SequenceTrack::fns().setInstrument.unwrap()(self.cptr(), instrument.cptr()) };
    self.instrument_ref = Some(InstrumentRef::from_ptr(instrument.cptr()))
  }


  /// Returns the length, in steps, of the track - ​that is, the step where the last note in the
  /// track ends.
  pub fn steps_count(&self) -> u32 {
    unsafe { SequenceTrack::fns().getLength.unwrap()(self.cptr()) }
  }

  /// Adds a single note to the track.
  pub fn add_note(&mut self, step: u32, note: TrackNote) {
    unsafe {
      SequenceTrack::fns().addNoteEvent.unwrap()(
        self.cptr(),
        step,
        note.length,
        note.midi_note,
        note.velocity,
      )
    }
  }
  /// Removes the event at `step` playing `midi_note`.
  pub fn remove_note_event(&mut self, step: u32, midi_note: f32) {
    unsafe { SequenceTrack::fns().removeNoteEvent.unwrap()(self.cptr(), step, midi_note) }
  }
  /// Remove all notes from the track.
  pub fn remove_all_notes(&mut self) {
    unsafe { SequenceTrack::fns().clearNotes.unwrap()(self.cptr()) }
  }

  pub fn notes_at_step(&self, step: u32) -> impl Iterator<Item = TrackNote> {
    let mut v = Vec::new();
    let first_index = unsafe { SequenceTrack::fns().getIndexForStep.unwrap()(self.cptr(), step) };
    for index in first_index.. {
      let mut out_step = 0;
      let mut length = 0;
      let mut midi_note = 0.0;
      let mut velocity = 0.0;
      let r = unsafe {
        SequenceTrack::fns().getNoteAtIndex.unwrap()(
          self.cptr(),
          index,
          &mut out_step,
          &mut length,
          &mut midi_note,
          &mut velocity,
        )
      };
      if r == 0 || out_step != step {
        break;
      }
      v.push(TrackNote {
        length,
        midi_note,
        velocity,
      });
    }
    v.into_iter()
  }

  pub fn notes(&self) -> impl Iterator<Item = TrackNote> {
    let mut v = Vec::new();
    for index in 0.. {
      let mut out_step = 0;
      let mut length = 0;
      let mut midi_note = 0.0;
      let mut velocity = 0.0;
      let r = unsafe {
        SequenceTrack::fns().getNoteAtIndex.unwrap()(
          self.cptr(),
          index,
          &mut out_step,
          &mut length,
          &mut midi_note,
          &mut velocity,
        )
      };
      if r == 0 {
        break;
      }
      v.push(TrackNote {
        length,
        midi_note,
        velocity,
      });
    }
    v.into_iter()
  }

  pub fn control_signal_count(&self) -> i32 {
    unsafe { SequenceTrack::fns().getControlSignalCount.unwrap()(self.cptr()) }
  }
  // TODO: Do something to expose the Control objects here, like an iterator or a slice.
  pub fn control_signal_at_index(&self, index: i32) {
    unsafe { SequenceTrack::fns().getControlSignal.unwrap()(self.cptr(), index) };
  }
  pub fn clear_control_signals(&mut self) {
    unsafe { SequenceTrack::fns().clearControlEvents.unwrap()(self.cptr()) }
  }

  // TODO: getSignalForController in a future update.
  // https://devforum.play.date/t/c-api-sequencetrack-is-missing-addcontrolsignal/4508

  /// Returns the maximum number of notes simultaneously active in the track.
  ///
  /// Known bug: this currently only works for midi files.
  pub fn polyphony(&self) -> i32 {
    unsafe { SequenceTrack::fns().getPolyphony.unwrap()(self.cptr()) }
  }

  /// Returns the current number of active notes in the track.
  pub fn active_notes_count(&self) -> i32 {
    unsafe { SequenceTrack::fns().activeVoiceCount.unwrap()(self.cptr()) }
  }

  /// Mutes the track.
  pub fn set_muted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr(), true as i32) }
  }
  /// Unmutes the track.
  pub fn set_unmuted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr(), false as i32) }
  }
}

/// An immutable unowned `SequenceTrack`.
pub struct UnownedSequenceTrack<'a> {
  tref: SequenceTrackRef,
  _marker: PhantomData<&'a u8>,
}
impl UnownedSequenceTrack<'_> {
  pub(crate) fn new(ptr: *mut CSequenceTrack, inst_ref: Option<InstrumentRef>) -> Self {
    UnownedSequenceTrack {
      tref: SequenceTrackRef::new(ptr, inst_ref),
      _marker: PhantomData,
    }
  }
}
impl core::ops::Deref for UnownedSequenceTrack<'_> {
  type Target = SequenceTrackRef;

  fn deref(&self) -> &Self::Target {
    &self.tref
  }
}
impl AsRef<SequenceTrackRef> for UnownedSequenceTrack<'_> {
  fn as_ref(&self) -> &SequenceTrackRef {
    self
  }
}
impl core::borrow::Borrow<SequenceTrackRef> for UnownedSequenceTrack<'_> {
  fn borrow(&self) -> &SequenceTrackRef {
    self
  }
}

/// A mutable unowned `SequenceTrack`.
pub struct UnownedSequenceTrackMut<'a> {
  tref: SequenceTrackRef,
  _marker: PhantomData<&'a u8>,
}
impl UnownedSequenceTrackMut<'_> {
  pub(crate) fn new(ptr: *mut CSequenceTrack, inst_ref: Option<InstrumentRef>) -> Self {
    UnownedSequenceTrackMut {
      tref: SequenceTrackRef::new(ptr, inst_ref),
      _marker: PhantomData,
    }
  }
}
impl core::ops::Deref for UnownedSequenceTrackMut<'_> {
  type Target = SequenceTrackRef;

  fn deref(&self) -> &Self::Target {
    &self.tref
  }
}
impl core::ops::DerefMut for UnownedSequenceTrackMut<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.tref
  }
}
impl AsRef<SequenceTrackRef> for UnownedSequenceTrackMut<'_> {
  fn as_ref(&self) -> &SequenceTrackRef {
    self
  }
}
impl AsMut<SequenceTrackRef> for UnownedSequenceTrackMut<'_> {
  fn as_mut(&mut self) -> &mut SequenceTrackRef {
    self
  }
}
impl core::borrow::Borrow<SequenceTrackRef> for UnownedSequenceTrackMut<'_> {
  fn borrow(&self) -> &SequenceTrackRef {
    self
  }
}
impl core::borrow::BorrowMut<SequenceTrackRef> for UnownedSequenceTrackMut<'_> {
  fn borrow_mut(&mut self) -> &mut SequenceTrackRef {
    self
  }
}
pub struct SequenceTrack {
  tref: SequenceTrackRef,
}
impl SequenceTrack {
  pub fn new() -> SequenceTrack {
    let ptr = unsafe { Self::fns().newTrack.unwrap()() };
    let instrument = Instrument::new();
    unsafe { Self::fns().setInstrument.unwrap()(ptr, instrument.cptr()) };
    SequenceTrack {
      tref: SequenceTrackRef::new(ptr, Some(InstrumentRef::from_ptr(instrument.cptr()))),
    }
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_track {
    unsafe { &*CApiState::get().csound.track }
  }
}

impl Drop for SequenceTrack {
  fn drop(&mut self) {
    unsafe { Self::fns().freeTrack.unwrap()(self.ptr.as_ptr()) }
  }
}

impl core::ops::Deref for SequenceTrack {
  type Target = SequenceTrackRef;

  fn deref(&self) -> &Self::Target {
    &self.tref
  }
}
impl core::ops::DerefMut for SequenceTrack {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.tref
  }
}
impl AsRef<SequenceTrackRef> for SequenceTrack {
  fn as_ref(&self) -> &SequenceTrackRef {
    self
  }
}
impl AsMut<SequenceTrackRef> for SequenceTrack {
  fn as_mut(&mut self) -> &mut SequenceTrackRef {
    self
  }
}
impl core::borrow::Borrow<SequenceTrackRef> for SequenceTrack {
  fn borrow(&self) -> &SequenceTrackRef {
    self
  }
}
impl core::borrow::BorrowMut<SequenceTrackRef> for SequenceTrack {
  fn borrow_mut(&mut self) -> &mut SequenceTrackRef {
    self
  }
}
