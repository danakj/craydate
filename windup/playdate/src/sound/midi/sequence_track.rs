use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::Instrument;
use super::sequence::Sequence;
use super::sequence_track_control::SequenceTrackControl;
use super::track_note::{ResolvedTrackNote, TrackNote};
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
  instrument: NonNull<Instrument>,
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
      // We store a mutable pointer here, but `SequenceTrack` will not mutate it. Only
      // `SequenceTrackMut` would do so, and if this type is being constructed from there, then
      // `SequenceTrackMut` did receive the same pointer as a `*mut Instrument`.
      instrument: NonNull::new(instrument as *mut _).unwrap(),
      _marker: PhantomData,
    }
  }

  /// Returns the track's index in its `Sequence`.
  pub fn index(&self) -> u32 {
    self.index
  }

  /// Gets the `Instrument` assigned to the track.
  pub fn instrument(&self) -> &'a Instrument {
    unsafe { &*self.instrument.as_ptr() }
  }

  /// Returns the length, in steps, of the track - ​that is, the step where the last note in the
  /// track ends.
  pub fn steps_count(&self) -> u32 {
    // getLength() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { SequenceTrack::fns().getLength.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns the maximum number of notes simultaneously active in the track.
  ///
  /// Known bug: this currently only works for midi files.
  pub fn polyphony(&self) -> i32 {
    // polyphony() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { SequenceTrack::fns().getPolyphony.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns the current number of active notes in the track.
  pub fn active_notes_count(&self) -> i32 {
    // activeVoiceCount() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { SequenceTrack::fns().activeVoiceCount.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns an iterator over all notes, as `ResolvedTrackNote`, in the track that start between
  /// `start_step` and `end_step`, inclusive.
  pub fn notes_in_step_range(
    &self,
    start_step: u32,
    end_step: u32,
  ) -> impl Iterator<Item = ResolvedTrackNote> {
    let mut v = Vec::new();
    // getIndexForStep() takes a mutable pointer but doesn't mutate any visible state.
    let first_index =
      unsafe { SequenceTrack::fns().getIndexForStep.unwrap()(self.cptr() as *mut _, start_step) };
    for index in first_index.. {
      let mut out_step = 0;
      let mut length = 0;
      let mut midi_note = 0.0;
      let mut velocity = 0.0;
      let r = unsafe {
        // getNoteAtIndex() takes a mutable pointer but doesn't mutate any visible state.
        SequenceTrack::fns().getNoteAtIndex.unwrap()(
          self.cptr() as *mut _,
          index,
          &mut out_step,
          &mut length,
          &mut midi_note,
          &mut velocity,
        )
      };
      if r == 0 || out_step > end_step {
        break;
      }
      v.push(ResolvedTrackNote {
        length,
        midi_note: midi_note as u8,
        velocity: velocity.into(),
      });
    }
    v.into_iter()
  }

  /// Returns an iterator over all notes, as `ResolvedTrackNote`, in the track.
  pub fn notes(&self) -> impl Iterator<Item = ResolvedTrackNote> {
    let mut v = Vec::new();
    for index in 0.. {
      let mut out_step = 0;
      let mut length = 0;
      let mut midi_note = 0.0;
      let mut velocity = 0.0;
      let r = unsafe {
        // getNoteAtIndex() takes a mutable pointer but doesn't mutate any visible state.
        SequenceTrack::fns().getNoteAtIndex.unwrap()(
          self.cptr() as *mut _,
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
      v.push(ResolvedTrackNote {
        length,
        midi_note: midi_note as u8,
        velocity: velocity.into(),
      });
    }
    v.into_iter()
  }

  /// Returns the number of `Control` signals in the track.
  pub fn signals_count(&self) -> usize {
    // getControlSignalCount() takes a mutable pointer but doesn't change any visible state.
    unsafe { SequenceTrack::fns().getControlSignalCount.unwrap()(self.cptr() as *mut _) as usize }
  }

  /// Returns an iterator over all `Control` signals in the track. The `Control` objects are
  /// owned inside `SequenceTrack` and are returned as the `SequenceTrackControl` type.
  pub fn signals(&self) -> impl Iterator<Item = SequenceTrackControl> {
    let mut v = Vec::new();
    // getControlSignalCount() takes a mutable pointer but doesn't change any visible state.
    let count =
      unsafe { SequenceTrack::fns().getControlSignalCount.unwrap()(self.cptr() as *mut _) };
    for i in 0..count {
      // getControlSignal() takes a mutable pointer but doesn't change any visible state.
      let ptr = unsafe { SequenceTrack::fns().getControlSignal.unwrap()(self.cptr() as *mut _, i) };
      v.push(SequenceTrackControl::from_ptr(NonNull::new(ptr).unwrap()));
    }
    v.into_iter()
  }

  // Returns the `SequenceTrackControl` (which is an track-owned `Control`) for the MIDI controller
  // number `midi_controller`. If the `Control` does not exist, then `None` is returned.
  pub fn signal_for_midi_controller(&self, midi_controller: i32) -> Option<SequenceTrackControl> {
    let ptr = unsafe {
      // getSignalForController() when create=false takes a mutable pointer but does not change any
      // visible state.
      Self::fns().getSignalForController.unwrap()(
        self.cptr() as *mut _,
        midi_controller,
        false as i32,
      )
    };
    Some(SequenceTrackControl::from_ptr(NonNull::new(ptr)?))
  }

  pub(crate) fn cptr(&self) -> *const CSequenceTrack {
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
  sequence_ptr: NonNull<Sequence>,
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
      sequence_ptr: NonNull::new(sequence_ptr).unwrap(),
    }
  }

  unsafe fn sequence(&mut self) -> &'a mut Sequence {
    // SAFETY: Constructs a reference `&'a mut Sequence` that will not outlive the `Sequence` from
    // which this object was constructed, as we hold a borrow on it with lifetime `&mut 'a`. The
    // reference should not be held more than a single function or it may alias with another call.
    self.sequence_ptr.as_mut()
  }

  /// Gets the `Instrument` assigned to the track.
  pub fn instrument_mut(&mut self) -> &'a mut Instrument {
    unsafe { &mut *self.instrument.as_ptr() }
  }

  /// Adds a single note to the track, with a length specified in steps, not time.
  ///
  /// The speed that steps are played depends on the tempo of the sequence.
  pub fn add_note(&mut self, step: u32, note: TrackNote, length: u32) {
    unsafe {
      SequenceTrack::fns().addNoteEvent.unwrap()(
        self.cptr_mut(),
        step,
        length,
        note.midi_note as f32,
        note.velocity.into(),
      )
    }
  }
  /// Removes the event at `step` playing `midi_note`.
  pub fn remove_note_event(&mut self, step: u32, midi_note: f32) {
    unsafe { SequenceTrack::fns().removeNoteEvent.unwrap()(self.cptr_mut(), step, midi_note) }
  }
  /// Remove all notes from the track.
  pub fn remove_all_notes(&mut self) {
    unsafe { SequenceTrack::fns().clearNotes.unwrap()(self.cptr_mut()) }
  }

  /// Sets the `Instrument` assigned to the track, taking ownership of the instrument.
  pub fn set_instrument(&mut self, mut instrument: Instrument) {
    unsafe { SequenceTrack::fns().setInstrument.unwrap()(self.cptr_mut(), instrument.cptr_mut()) };
    // SAFETY: The `Sequence` reference has a lifetime `&'a mut`, so it will outlive `self` and the
    // `Sequence` borrowed by `self` as `&'a mut`. The `&mut Instrument` does not hold a reference
    // that would alias with the `&mut Sequence` (as seen by its lack of lifetime parameter).
    let seq = unsafe { self.sequence() };
    seq.set_track_instrument(self.index, instrument);
    let instrument: &mut Instrument = seq.track_instrument_mut(self.index);
    self.track.instrument = unsafe { NonNull::new_unchecked(instrument as *mut _) };
  }

  /// Mutes the track.
  pub fn set_muted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr_mut(), true as i32) }
  }
  /// Unmutes the track.
  pub fn set_unmuted(&mut self) {
    unsafe { SequenceTrack::fns().setMuted.unwrap()(self.cptr_mut(), false as i32) }
  }

  /// Remove all control signals from the track.
  pub fn clear_control_signals(&mut self) {
    unsafe { SequenceTrack::fns().clearControlEvents.unwrap()(self.cptr_mut()) }
  }

  // Create a `SequenceTrackControl` (which is an track-owned `Control`) for the MIDI controller
  // number `midi_controller`. If the `Control` already exists, then `None` is returned.
  pub fn create_signal_for_midi_controller(&mut self, midi_controller: i32) -> CreateSignalResult {
    let ptr = unsafe {
      SequenceTrack::fns().getSignalForController.unwrap()(
        self.cptr_mut(),
        midi_controller,
        false as i32,
      )
    };
    if !ptr.is_null() {
      CreateSignalResult::AlreadyExists(SequenceTrackControl::from_ptr(NonNull::new(ptr).unwrap()))
    } else {
      let ptr = unsafe {
        SequenceTrack::fns().getSignalForController.unwrap()(
          self.cptr_mut(),
          midi_controller,
          true as i32,
        )
      };
      CreateSignalResult::Created(SequenceTrackControl::from_ptr(NonNull::new(ptr).unwrap()))
    }
  }

  pub(crate) fn cptr_mut(&mut self) -> *mut CSequenceTrack {
    self.ptr.as_ptr()
  }
}

/// The result of trying to create a `Control` for a MIDI controller.
pub enum CreateSignalResult<'a> {
  Created(SequenceTrackControl<'a>),
  AlreadyExists(SequenceTrackControl<'a>),
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
