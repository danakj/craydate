use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::capi_state::CApiState;
use crate::ctypes::*;

pub struct TrackNote {
  length: u32,
  midi_note: f32,
  velocity: f32,
}

pub struct SequenceTrackRef {
  ptr: NonNull<CSequenceTrack>,
}
impl SequenceTrackRef {
  pub fn from_ptr(ptr: *mut CSequenceTrack) -> Self {
    SequenceTrackRef {
      ptr: NonNull::new(ptr).unwrap(),
    }
  }
  pub(crate) fn cptr(&self) -> *mut CSequenceTrack {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_track {
    unsafe { &*CApiState::get().csound.track }
  }

  // TODO: getInstrument
  // TODO: setInstrument

  /// Returns the length, in steps, of the track - ​that is, the step where the last note in the
  /// track ends.
  pub fn steps_count(&self) -> u32 {
    unsafe { Self::fns().getLength.unwrap()(self.cptr()) }
  }

  /// Adds a single note event to the track.
  pub fn add_note_event(&mut self, step: u32, note: TrackNote) {
    unsafe {
      Self::fns().addNoteEvent.unwrap()(
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
    unsafe { Self::fns().removeNoteEvent.unwrap()(self.cptr(), step, midi_note) }
  }
  /// Remove all notes from the track.
  pub fn remove_all_notes(&mut self) {
    unsafe { Self::fns().clearNotes.unwrap()(self.cptr()) }
  }

  pub fn notes_at_step(&self, step: u32) -> impl Iterator<Item = TrackNote> {
    let mut v = Vec::new();
    let first_index = unsafe { Self::fns().getIndexForStep.unwrap()(self.cptr(), step) };
    for index in first_index.. {
      let mut out_step = 0;
      let mut length = 0;
      let mut midi_note = 0.0;
      let mut velocity = 0.0;
      let r = unsafe {
        Self::fns().getNoteAtIndex.unwrap()(
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

  // TODO: Iterator over all notes.

  // TODO: Lots more functions here.
  
  // TODO: Missing addControlSignal() in the C API:
  // https://devforum.play.date/t/c-api-sequencetrack-is-missing-addcontrolsignal/4508
}

/// An immutable unowned `SequenceTrack`.
pub struct UnownedSequenceTrack<'a> {
  tref: SequenceTrackRef,
  _marker: PhantomData<&'a u8>,
}
impl UnownedSequenceTrack<'_> {
  pub(crate) fn from_ptr(ptr: *mut CSequenceTrack) -> Self {
    UnownedSequenceTrack {
      tref: SequenceTrackRef::from_ptr(ptr),
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
  pub(crate) fn from_ptr(ptr: *mut CSequenceTrack) -> Self {
    UnownedSequenceTrackMut {
      tref: SequenceTrackRef::from_ptr(ptr),
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
    SequenceTrack {
      tref: SequenceTrackRef::from_ptr(ptr),
    }
  }

  fn fns() -> &'static playdate_sys::playdate_sound_track {
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
