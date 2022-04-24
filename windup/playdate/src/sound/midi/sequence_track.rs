use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::Instrument;
use super::sequence::Sequence;
use super::track_note::TrackNote;
use crate::capi_state::CApiState;
use crate::ctypes::*;

pub struct SequenceTrackRef<'a> {
  ptr: NonNull<CSequenceTrack>,
  index: u32,
  instrument: Option<*mut Instrument>,
  _marker: PhantomData<&'a Sequence>,
}
impl<'a> SequenceTrackRef<'a> {
  pub fn new(ptr: *mut CSequenceTrack, index: u32, instrument: Option<*mut Instrument>) -> Self {
    SequenceTrackRef {
      ptr: NonNull::new(ptr).unwrap(),
      index,
      instrument,
      _marker: PhantomData,
    }
  }
  pub(crate) fn cptr(&self) -> *mut CSequenceTrack {
    self.ptr.as_ptr()
  }

  /// Returns the track's index in its `Sequence`.
  pub fn index(&self) -> u32 {
    self.index
  }

  /// Gets the `Instrument` assigned to the track.
  pub fn instrument(&self) -> Option<&'a Instrument> {
    self.instrument.map(|p| unsafe { &*p })
  }
  /// Gets the `Instrument` assigned to the track.
  pub fn instrument_mut(&mut self) -> Option<&'a mut Instrument> {
    self.instrument.map(|p| unsafe { &mut *p })
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
        note.midi_note as f32,
        note.velocity.into(),
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
        midi_note: midi_note as u8,
        velocity: velocity.into(),
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
        midi_note: midi_note as u8,
        velocity: velocity.into(),
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
pub struct SequenceTrack<'a> {
  tref: SequenceTrackRef<'a>,
  _seq: &'a Sequence,
}
impl<'a> SequenceTrack<'a> {
  pub(crate) fn new<'b>(ptr: *mut CSequenceTrack, index: u32, seq: &'a Sequence) -> Self {
    // SAFETY: This type only gives const access to the SequenceTrackRef so the pointee is not
    // mutated.
    let instrument = seq.track_instrument(index) as *const _ as *mut _;
    SequenceTrack {
      tref: SequenceTrackRef::new(ptr, index, Some(instrument)),
      _seq: seq,
    }
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_track {
    unsafe { &*CApiState::get().csound.track }
  }
}
impl<'a> core::ops::Deref for SequenceTrack<'a> {
  type Target = SequenceTrackRef<'a>;

  fn deref(&self) -> &Self::Target {
    &self.tref
  }
}
impl<'a> AsRef<SequenceTrackRef<'a>> for SequenceTrack<'a> {
  fn as_ref(&self) -> &SequenceTrackRef<'a> {
    self
  }
}

/// A mutable unowned `SequenceTrack`.
pub struct SequenceTrackMut<'a> {
  tref: SequenceTrackRef<'a>,
  seq: *mut Sequence,
}
impl<'a> SequenceTrackMut<'a> {
  pub(crate) fn new(ptr: *mut CSequenceTrack, index: u32, seq: *mut Sequence) -> Self {
    let instrument = unsafe { (*seq).track_instrument_mut(index) as *mut Instrument };
    SequenceTrackMut {
      tref: SequenceTrackRef::new(ptr, index, Some(instrument)),
      seq,
    }
  }

  unsafe fn sequence(&self) -> &'a mut Sequence {
    // SAFETY: Constructs a reference `&'a mut Sequence` that will not outlive the `Sequence` from
    // which this object was constructed, as we hold a borrow on it with lifetime `&mut 'a`.
    &mut *self.seq
  }

  /// Sets the `Instrument` assigned to the track, taking ownership of the instrument.
  pub fn set_instrument(&mut self, instrument: Instrument) {
    unsafe { SequenceTrack::fns().setInstrument.unwrap()(self.cptr(), instrument.cptr()) };
    // SAFETY: The `Sequence` reference has a lifetime `&'a mut`, so it will outlive `self` and the
    // `Sequence` borrowed by `self` as `&'a mut`. The `&mut Instrument` does not hold a reference
    // that would alias with the `&mut Sequence` (as seen by its lack of lifetime parameter).
    let seq = unsafe { self.sequence() };
    seq.set_track_instrument(self.index, instrument);
    let iref: &mut Instrument = seq.track_instrument_mut(self.index);
    self.tref.instrument = Some(iref);
  }
}

impl<'a> core::ops::Deref for SequenceTrackMut<'a> {
  type Target = SequenceTrackRef<'a>;

  fn deref(&self) -> &Self::Target {
    &self.tref
  }
}
impl core::ops::DerefMut for SequenceTrackMut<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.tref
  }
}
impl<'a> AsRef<SequenceTrackRef<'a>> for SequenceTrackMut<'a> {
  fn as_ref(&self) -> &SequenceTrackRef<'a> {
    self
  }
}
impl<'a> AsMut<SequenceTrackRef<'a>> for SequenceTrackMut<'a> {
  fn as_mut(&mut self) -> &mut SequenceTrackRef<'a> {
    self
  }
}
