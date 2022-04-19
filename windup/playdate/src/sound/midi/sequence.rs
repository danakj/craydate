use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::Instrument;
use super::super::SoundCompletionCallback;
use super::sequence_track::{SequenceTrack, SequenceTrackMut};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;

/// Represents a MIDI music file, as a collection of `SequenceTrack`s that can be played together.
pub struct Sequence {
  ptr: NonNull<CSoundSequence>,
  finished_callback: Option<RegisteredCallback>,

  // Holds ownership of user-created tracks. Loading a MIDI file  generates Playdate-owned tracks
  // which are not represented here.
  user_created_tracks: Vec<NonNull<CSequenceTrack>>,
  // The set of instruments attached to tracks. Some of the tracks are owned by Playdate, and some
  // are owned by the this Sequence type. But all instruments are owned by this Sequence.
  instruments: BTreeMap<u32, Instrument>,
}
impl Sequence {
  fn from_ptr(ptr: *mut CSoundSequence) -> Self {
    Sequence {
      ptr: NonNull::new(ptr).unwrap(),
      finished_callback: None,
      user_created_tracks: Vec::new(),
      instruments: BTreeMap::new(),
    }
  }

  /// Constructs a new `Sequence`, which is a set of `SequenceTrack`s that can be played together.
  pub(crate) fn new() -> Self {
    let ptr = unsafe { Self::fns().newSequence.unwrap()() };
    Self::from_ptr(ptr)
  }

  /// Loads the midi file at `path` and constructs a `Sequence` from it.
  ///
  /// Returns an `Error::LoadMidiFileError` if loading the file did not succeed. No further
  /// information about why the load failed is available.
  pub fn from_midi_file(path: &str) -> Result<Self, Error> {
    let mut seq = Self::new();
    let r = unsafe {
      Self::fns().loadMidiFile.unwrap()(seq.cptr(), path.to_null_terminated_utf8().as_ptr())
    };
    match r {
      0 => Err(Error::LoadMidiFileError),
      _ => {
        seq.create_instrument_for_each_track();
        Ok(seq)
      }
    }
  }

  fn create_instrument_for_each_track(&mut self) {
    let mut instruments = BTreeMap::new();
    for t in self.tracks() {
      if !self.instruments.contains_key(&t.index()) {
        assert!(unsafe { SequenceTrack::fns().getInstrument.unwrap()(t.cptr()) }.is_null());
        instruments.insert(t.index(), Instrument::new());
      }
    }
    self.instruments = instruments;
  }

  pub(crate) fn set_track_instrument(&mut self, index: u32, instrument: Instrument) {
    self.instruments.insert(index, instrument);
  }
  pub(crate) fn track_instrument(&self, index: u32) -> &Instrument {
    self.instruments.get(&index).unwrap()
  }
  pub(crate) fn track_instrument_mut(&mut self, index: u32) -> &mut Instrument {
    self.instruments.get_mut(&index).unwrap()
  }

  /// Starts playing the sequence.
  ///
  /// The `finished_callback` is an optional closure to be called when the sequence finishes playing
  /// or is stopped.
  pub fn play<'a, T, F: Fn(T) + 'static>(
    &mut self,
    finished_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    let func = finished_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.cptr() as usize;
      let (func, reg) = callbacks.add_sequence_finished(key, cb);
      self.finished_callback = Some(reg);
      Some(func)
    });
    unsafe { Self::fns().play.unwrap()(self.cptr(), func, core::ptr::null_mut()) }
  }

  /// Stops playing the sequence.
  pub fn stop(&mut self) {
    unsafe { Self::fns().stop.unwrap()(self.cptr()) }
  }

  /// Sends a stop signal to all playing notes on all tracks.
  pub fn all_notes_off(&mut self) {
    unsafe { Self::fns().allNotesOff.unwrap()(self.cptr()) }
  }

  /// Returns if the sequence is currently playing.
  pub fn is_playing(&self) -> bool {
    unsafe { Self::fns().isPlaying.unwrap()(self.cptr()) != 0 }
  }

  /// Sets the current time in the sequence, in steps since the start of the MIDI file.
  ///
  /// Note that which step this moves the sequence to depends on the current tempo.
  pub fn set_current_step(&mut self, time: u32) {
    unsafe { Self::fns().setTime.unwrap()(self.cptr(), time) }
  }
  /// Gets the current time in the sequence, in steps since the start of the file.
  ///
  /// Note that which step this refers to depends on the current tempo.
  pub fn current_step(&self) -> u32 {
    unsafe { Self::fns().getTime.unwrap()(self.cptr()) }
  }

  /// Sets the tempo of the sequence, in steps per second.
  pub fn set_tempo(&mut self, steps_per_second: i32) {
    unsafe { Self::fns().setTempo.unwrap()(self.cptr(), steps_per_second) }
  }
  /// Gets the tempo of the sequence, in steps per second.
  pub fn tempo(&mut self) -> i32 {
    unsafe { Self::fns().getTempo.unwrap()(self.cptr()) }
  }

  /// Returns the length of the longest track in the sequence.
  ///
  /// See also `SequenceTrack::steps_count()`.
  pub fn steps_count(&self) -> u32 {
    unsafe { Self::fns().getLength.unwrap()(self.cptr()) }
  }

  /// Returns the number of tracks in the sequence.
  pub fn tracks_count(&self) -> u32 {
    let c = unsafe { Self::fns().getTrackCount.unwrap()(self.cptr()) };
    // getTrackCount() returns i32, but getTrackAtIndex takes u32. If anything, we could expect
    // getTrackCount() to change to u32 one day, so we'll cast to that instead of the other way.
    c as u32
  }

  /// Returns an iterator over all the tracks in the `Sequence`.
  pub fn tracks<'a>(&'a self) -> impl Iterator<Item = SequenceTrack> + 'a {
    SequenceTrackIter {
      seq: self,
      next: 0,
      count: self.tracks_count(),
    }
  }
  /// Returns a mutable iterator over all the tracks in the `Sequence`.
  pub fn tracks_mut<'a>(&'a mut self) -> impl Iterator<Item = SequenceTrackMut<'a>> + 'a {
    SequenceTrackIterMut {
      seq: self,
      next: 0,
      count: self.tracks_count(),
      _marker: PhantomData,
    }
  }

  pub fn create_track_at_index(&mut self, index: u32) -> SequenceTrackMut<'_> {
    let track_ptr = NonNull::new(unsafe { SequenceTrack::fns().newTrack.unwrap()() }).unwrap();
    unsafe { Sequence::fns().setTrackAtIndex.unwrap()(self.cptr(), track_ptr.as_ptr(), index) };
    self.instruments.insert(index, Instrument::new());
    SequenceTrackMut::new(track_ptr.as_ptr(), index, self)
  }
  pub fn track_at_index(&self, index: u32) -> Option<SequenceTrack> {
    if self.instruments.contains_key(&index) {
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.cptr(), index) };
      assert!(!track_ptr.is_null());
      Some(SequenceTrack::new(track_ptr, index, self))
    } else {
      None
    }
  }
  pub fn track_at_index_mut(&mut self, index: u32) -> Option<SequenceTrackMut<'_>> {
    if self.instruments.contains_key(&index) {
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.cptr(), index) };
      assert!(!track_ptr.is_null());
      Some(SequenceTrackMut::new(track_ptr, index, self))
    } else {
      None
    }
  }

  /// Sets the looping range of the sequence.
  ///
  /// If loops is 0, the loop repeats endlessly.
  pub fn set_loops(&mut self, start_step: u32, end_step: u32, count: i32) {
    // TODO: The step numbers should be u32 but Playdate has them as `int`.
    unsafe { Self::fns().setLoops.unwrap()(self.cptr(), start_step as i32, end_step as i32, count) }
  }

  fn cptr(&self) -> *mut CSoundSequence {
    self.ptr.as_ptr()
  }
  fn fns() -> &'static playdate_sys::playdate_sound_sequence {
    unsafe { &*CApiState::get().csound.sequence }
  }
}

impl Drop for Sequence {
  fn drop(&mut self) {
    // The instruments will be dropped after the sequence-owned tracks that refer to them.
    unsafe { Self::fns().freeSequence.unwrap()(self.cptr()) }
    // The instruments will be dropped after the sequence-owned tracks that refer to them.
    for ptr in self.user_created_tracks.drain(..) {
      unsafe { SequenceTrack::fns().freeTrack.unwrap()(ptr.as_ptr()) }
    }
  }
}

struct SequenceTrackIter<'a> {
  seq: &'a Sequence,
  next: u32,
  count: u32,
}
impl<'a> Iterator for SequenceTrackIter<'a> {
  type Item = SequenceTrack<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count == 0 {
      None
    } else {
      loop {
        let index = self.next;
        self.next += 1;
        let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.seq.cptr(), index) };
        if !track_ptr.is_null() {
          self.count -= 1;
          return Some(SequenceTrack::new(track_ptr, index, self.seq));
        }
      }
    }
  }
}

struct SequenceTrackIterMut<'a> {
  seq: *mut Sequence,
  next: u32,
  count: u32,
  _marker: PhantomData<&'a Sequence>,
}
impl<'a> Iterator for SequenceTrackIterMut<'a> {
  type Item = SequenceTrackMut<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count == 0 {
      None
    } else {
      loop {
        let index = self.next;
        self.next += 1;
        // SAFETY: Reborrow the `Sequence` pointer to construct a reference without a lifetime
        // bounded by `self`. We only need the reference to `Sequence` to live as long as `'a` which
        // is represented by the lifetime parameter on the output `Item` type.
        let seq = unsafe { &mut *(self.seq) };
        let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(seq.cptr(), index) };
        if !track_ptr.is_null() {
          self.count -= 1;
          return Some(SequenceTrackMut::new(track_ptr, index, seq));
        }
      }
    }
  }
}
