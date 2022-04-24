use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::Instrument;
use super::sequence::Sequence;
use super::track_note::TrackNote;
use crate::capi_state::CApiState;
use crate::ctypes::*;

/// A `SequenceTrack` plays (multiple at a time) notes on an `Instrument` as part of a full
/// `Sequence`, which represents a MIDI file.
///
/// The data in a `SequenceTrack` is owned by the `Sequence`, so this represents a borrow of that
/// data.
pub struct SequenceTrack<'a> {
  ptr: NonNull<CSequenceTrack>,
  index: u32,
  instrument: Option<*mut Instrument>,
  _marker: PhantomData<&'a Sequence>,
}
impl<'a> SequenceTrack<'a> {
  pub(crate) fn new<'b>(
    ptr: *mut CSequenceTrack,
    index: u32,
    instrument: *const Instrument,
  ) -> Self {
    SequenceTrack {
      ptr: NonNull::new(ptr).unwrap(),
      index,
      // We store a mutable pointer here, but SequenceTrack will not mutate it. Only
      // SequenceTrackMut would do so, and it received the same pointer as a *mut Instrument.
      instrument: Some(instrument as *mut _),
      _marker: PhantomData,
    }
  }

  /// Returns the track's index in its `Sequence`.
  pub fn index(&self) -> u32 {
    self.index
  }

  /// Gets the `Instrument` assigned to the track.
  pub fn instrument(&self) -> Option<&'a Instrument> {
    self.instrument.map(|p| unsafe { &*p })
  }

  /// Returns the length, in steps, of the track - ​that is, the step where the last note in the
  /// track ends.
  pub fn steps_count(&self) -> u32 {
    unsafe { SequenceTrack::fns().getLength.unwrap()(self.cptr()) }
  }

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

  /// Returns an iterator over all `TrackNote`s in the track that start at the given `step`.
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

  /// Returns an iterator over all `TrackNote`s in the track.
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

  // TODO: Replace this function with a slice or iterator accessor of the signals.
  // /// Returns the number of control signals in the track.
  // pub fn control_signal_count(&self) -> i32 {
  //   unsafe { SequenceTrack::fns().getControlSignalCount.unwrap()(self.cptr()) }
  // }

  // TODO: getControlSignal

  // TODO: getSignalForController (with `create = false`)

  pub(crate) fn cptr(&self) -> *mut CSequenceTrack {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound_track {
    unsafe { &*CApiState::get().csound.track }
  }
}

/// A `SequenceTrackMut` plays (multiple at a time) notes on an `Instrument` as part of a full
/// `Sequence`, which represents a MIDI file.
///
/// The data in a `SequenceTrackMut` is owned by the `Sequence`, so this represents a borrow of that
/// data.
pub struct SequenceTrackMut<'a> {
  track: SequenceTrack<'a>,
  sequence_ptr: *mut Sequence,
}
impl<'a> SequenceTrackMut<'a> {
  pub(crate) fn new(
    ptr: *mut CSequenceTrack,
    index: u32,
    sequence_ptr: *mut Sequence,
    instrument: *mut Instrument,
  ) -> Self {
    SequenceTrackMut {
      track: SequenceTrack::new(ptr, index, instrument as *const _),
      sequence_ptr,
    }
  }

  unsafe fn sequence(&self) -> &'a mut Sequence {
    // SAFETY: Constructs a reference `&'a mut Sequence` that will not outlive the `Sequence` from
    // which this object was constructed, as we hold a borrow on it with lifetime `&mut 'a`. The
    // reference should not be held more than a single function or it may alias with another call.
    &mut *self.sequence_ptr
  }

  /// Gets the `Instrument` assigned to the track.
  pub fn instrument_mut(&mut self) -> Option<&'a mut Instrument> {
    self.instrument.map(|p| unsafe { &mut *p })
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

  /// Sets the `Instrument` assigned to the track, taking ownership of the instrument.
  pub fn set_instrument(&mut self, instrument: Instrument) {
    unsafe { SequenceTrack::fns().setInstrument.unwrap()(self.cptr(), instrument.cptr()) };
    // SAFETY: The `Sequence` reference has a lifetime `&'a mut`, so it will outlive `self` and the
    // `Sequence` borrowed by `self` as `&'a mut`. The `&mut Instrument` does not hold a reference
    // that would alias with the `&mut Sequence` (as seen by its lack of lifetime parameter).
    let seq = unsafe { self.sequence() };
    seq.set_track_instrument(self.index, instrument);
    let iref: &mut Instrument = seq.track_instrument_mut(self.index);
    self.track.instrument = Some(iref);
  }

  /// Mutes the track.
  pub fn set_muted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr(), true as i32) }
  }
  /// Unmutes the track.
  pub fn set_unmuted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr(), false as i32) }
  }

  /// Remove all control signals from the track.
  pub fn clear_control_signals(&mut self) {
    unsafe { SequenceTrack::fns().clearControlEvents.unwrap()(self.cptr()) }
  }

  // TODO: getSignalForController (with `create = true`)
}

impl<'a> core::ops::Deref for SequenceTrackMut<'a> {
  type Target = SequenceTrack<'a>;

  fn deref(&self) -> &Self::Target {
    &self.track
  }
}
impl<'a> AsRef<SequenceTrack<'a>> for SequenceTrackMut<'a> {
  fn as_ref(&self) -> &SequenceTrack<'a> {
    self
  }
}
