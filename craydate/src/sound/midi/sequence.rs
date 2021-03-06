use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::Instrument;
use super::super::SoundCompletionCallback;
use super::sequence_track::{SequenceTrack, SequenceTrackMut};
use crate::callback_builder::Constructed;
use crate::callbacks::RegisteredCallback;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;

/// Represents a MIDI music file, as a collection of `SequenceTrack`s that can be played together.
pub struct Sequence {
  ptr: NonNull<CSoundSequence>,
  finished_callback: Option<RegisteredCallback>,

  // Holds ownership of user-created tracks. Loading a MIDI file generates Playdate-owned tracks
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
      Self::fns().loadMidiFile.unwrap()(seq.cptr_mut(), path.to_null_terminated_utf8().as_ptr())
    };
    match r {
      0 => Err(Error::LoadMidiFileError),
      _ => {
        seq.create_instrument_for_each_track();
        Ok(seq)
      }
    }
  }

  /// Create an instrument for each track that doesn't have one set yet, so that all tracks in the
  /// `Sequence` always have an `Instrument`.
  fn create_instrument_for_each_track(&mut self) {
    let mut instruments = BTreeMap::new();
    let mut count = self.tracks_count();
    let mut index = 0;
    while count > 0 {
      // The track indices may not be contiguous, so we have to look for which indices don't have a
      // null track.
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.cptr_mut(), index) };
      if !track_ptr.is_null() {
        count -= 1;
        if !self.instruments.contains_key(&index) {
          assert!(unsafe { SequenceTrack::fns().getInstrument.unwrap()(track_ptr) }.is_null());
          let mut instrument = Instrument::new();
          unsafe { SequenceTrack::fns().setInstrument.unwrap()(track_ptr, instrument.cptr_mut()) };
          instruments.insert(index, instrument);
        }
      }
      index += 1;
    }
    self.instruments = instruments;
  }

  /// Called from `SequenceTrack`, where an `Instrument` can be set on it. This holds ownership of
  /// that `Instrument`.
  pub(crate) fn set_track_instrument(&mut self, index: u32, instrument: Instrument) {
    self.instruments.insert(index, instrument);
  }
  /// Gives access to the `Instrument` of a `SequenceTrack` from the `SequenceTrack`.
  pub(crate) fn track_instrument(&self, index: u32) -> &Instrument {
    self.instruments.get(&index).unwrap()
  }
  /// Gives access to the `Instrument` of a `SequenceTrack` from the `SequenceTrack`.
  pub(crate) fn track_instrument_mut(&mut self, index: u32) -> &mut Instrument {
    self.instruments.get_mut(&index).unwrap()
  }

  /// Starts playing the sequence.
  ///
  /// The `finished_callback` is an optional closure to be called when the sequence finishes playing
  /// or is stopped. It not `SoundCompletionCallback::none()`, the callback will be registered as a
  /// system event, and the application will be notified to run the callback via a
  /// `SystemEvent::Callback` event. When that occurs, the application's `Callbacks` object which
  /// was used to construct the `completion_callback` can be `run()` to execute the closure bound in
  /// the `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// sequence.play(SoundCompletionCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("playing done");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.run(12);
  ///   }
  /// }
  /// ```
  pub fn play<'a, T, F: Fn(T) + 'static>(
    &mut self,
    finished_callback: SoundCompletionCallback<'a, T, F, Constructed>,
  ) {
    self.finished_callback = None;
    let func = finished_callback.into_inner().and_then(|(callbacks, cb)| {
      let key = self.cptr_mut() as usize;
      let (func, reg) = callbacks.add_sequence_finished(key, cb);
      self.finished_callback = Some(reg);
      Some(func)
    });
    unsafe { Self::fns().play.unwrap()(self.cptr_mut(), func, core::ptr::null_mut()) }
  }

  /// Stops playing the sequence.
  pub fn stop(&mut self) {
    unsafe { Self::fns().stop.unwrap()(self.cptr_mut()) }
  }

  /// Sends a stop signal to all playing notes on all tracks.
  pub fn all_notes_off(&mut self) {
    unsafe { Self::fns().allNotesOff.unwrap()(self.cptr_mut()) }
  }

  /// Returns if the sequence is currently playing.
  pub fn is_playing(&self) -> bool {
    // isPlaying() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { Self::fns().isPlaying.unwrap()(self.cptr() as *mut _) != 0 }
  }

  /// Sets the current time in the sequence, in steps since the start of the MIDI file.
  ///
  /// Note that which step this moves the sequence to depends on the current tempo.
  pub fn set_current_step(&mut self, time: u32) {
    unsafe { Self::fns().setTime.unwrap()(self.cptr_mut(), time) }
  }
  /// Gets the current time in the sequence, in steps since the start of the file.
  ///
  /// Note that which step this refers to depends on the current tempo.
  pub fn current_step(&self) -> u32 {
    // getTime() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { Self::fns().getTime.unwrap()(self.cptr() as *mut _) }
  }

  /// Sets the tempo of the sequence, in steps per second.
  pub fn set_tempo(&mut self, steps_per_second: i32) {
    unsafe { Self::fns().setTempo.unwrap()(self.cptr_mut(), steps_per_second) }
  }
  /// Gets the tempo of the sequence, in steps per second.
  pub fn tempo(&mut self) -> i32 {
    // getTempo() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { Self::fns().getTempo.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns the length of the longest track in the sequence.
  ///
  /// See also `SequenceTrack::steps_count()`.
  pub fn steps_count(&self) -> u32 {
    // getLength() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { Self::fns().getLength.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns the number of tracks in the sequence.
  pub fn tracks_count(&self) -> u32 {
    // getTrackCount() takes a mutable pointer but doesn't mutate any visible state.
    let c = unsafe { Self::fns().getTrackCount.unwrap()(self.cptr() as *mut _) };
    // getTrackCount() returns i32, but getTrackAtIndex takes u32. If anything, we could expect
    // getTrackCount() to change to u32 one day, so we'll cast to that instead of the other way.
    c as u32
  }

  /// Returns an iterator over all the tracks in the `Sequence`.
  pub fn tracks<'a>(&'a self) -> impl Iterator<Item = SequenceTrack> + 'a {
    SequenceTrackIter {
      sequence: self,
      next_index: 0,
      count_left: self.tracks_count() as usize,
      count_total: self.tracks_count() as usize,
    }
  }
  /// Returns a mutable iterator over all the tracks in the `Sequence`.
  pub fn tracks_mut<'a>(&'a mut self) -> impl Iterator<Item = SequenceTrackMut<'a>> + 'a {
    SequenceTrackIterMut {
      sequence: NonNull::new(self).unwrap(),
      next_index: 0,
      count_left: self.tracks_count() as usize,
      count_total: self.tracks_count() as usize,
      _marker: PhantomData,
    }
  }

  /// Creates a new `SequenceTrack` at the given `index`, replacing an existing track if there was
  /// one.
  pub fn create_track_at_index(&mut self, index: u32) -> SequenceTrackMut<'_> {
    let track_ptr = unsafe { SequenceTrack::fns().newTrack.unwrap()() };
    assert!(!track_ptr.is_null());
    unsafe { Sequence::fns().setTrackAtIndex.unwrap()(self.cptr_mut(), track_ptr, index) };
    let mut instrument = Instrument::new();
    unsafe { SequenceTrack::fns().setInstrument.unwrap()(track_ptr, instrument.cptr_mut()) };
    self.instruments.insert(index, instrument);
    SequenceTrackMut::new(track_ptr, index, self, self.track_instrument_mut(index))
  }
  /// Gets the `SequenceTrack` at the given `index` if there is one. Otherwise, returns `None`.
  pub fn track_at_index(&self, index: u32) -> Option<SequenceTrack> {
    if self.instruments.contains_key(&index) {
      // getTrackAtIndex() takes a mutable pointer but doesn't mutate any visible state.
      let track_ptr =
        unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.cptr() as *mut _, index) };
      assert!(!track_ptr.is_null());
      Some(SequenceTrack::new(
        track_ptr,
        index,
        self.track_instrument(index),
      ))
    } else {
      None
    }
  }
  /// Gets the `SequenceTrack` at the given `index` if there is one. Otherwise, returns `None`.
  pub fn track_at_index_mut(&mut self, index: u32) -> Option<SequenceTrackMut<'_>> {
    if self.instruments.contains_key(&index) {
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.cptr_mut(), index) };
      assert!(!track_ptr.is_null());
      Some(SequenceTrackMut::new(
        track_ptr,
        index,
        self,
        self.track_instrument_mut(index),
      ))
    } else {
      None
    }
  }

  /// Sets the looping range of the sequence.
  ///
  /// If loops is 0, the loop repeats endlessly.
  pub fn set_loops(&mut self, start_step: u32, end_step: u32, count: i32) {
    // BUG: The step numbers should be u32 but the Playdate C Api has them as `int`:
    // <https://devforum.play.date/t/playdate-sound-sequence-setloops-takes-int-but-should-take-uint32-t/4980>
    unsafe {
      Self::fns().setLoops.unwrap()(self.cptr_mut(), start_step as i32, end_step as i32, count)
    }
  }

  pub(crate) fn cptr(&self) -> *const CSoundSequence {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CSoundSequence {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_sound_sequence {
    unsafe { &*CApiState::get().csound.sequence }
  }
}

impl Drop for Sequence {
  fn drop(&mut self) {
    // The instruments will be dropped after the sequence-owned tracks that refer to them.
    unsafe { Self::fns().freeSequence.unwrap()(self.cptr_mut()) }
    // The instruments will be dropped after the sequence-owned tracks that refer to them.
    for ptr in self.user_created_tracks.drain(..) {
      unsafe { SequenceTrack::fns().freeTrack.unwrap()(ptr.as_ptr()) }
    }
  }
}

struct SequenceTrackIter<'a> {
  sequence: &'a Sequence,
  next_index: u32,
  count_left: usize,
  count_total: usize,
}
impl<'a> Iterator for SequenceTrackIter<'a> {
  type Item = SequenceTrack<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count_left == 0 {
      None
    } else {
      loop {
        let index = self.next_index;
        self.next_index += 1;
        let track_ptr = unsafe {
          // getTrackAtIndex() takes a mutable pointer but doesn't mutate any visible state.
          Sequence::fns().getTrackAtIndex.unwrap()(self.sequence.cptr() as *mut _, index)
        };
        if !track_ptr.is_null() {
          self.count_left -= 1;
          return Some(SequenceTrack::new(
            track_ptr,
            index,
            self.sequence.track_instrument(index),
          ));
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.count_total, Some(self.count_total))
  }
}
impl ExactSizeIterator for SequenceTrackIter<'_> {}
impl core::iter::FusedIterator for SequenceTrackIter<'_> {}

struct SequenceTrackIterMut<'a> {
  sequence: NonNull<Sequence>,
  next_index: u32,
  count_left: usize,
  count_total: usize,
  _marker: PhantomData<&'a Sequence>,
}
impl<'a> Iterator for SequenceTrackIterMut<'a> {
  type Item = SequenceTrackMut<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count_left == 0 {
      None
    } else {
      loop {
        let index = self.next_index;
        self.next_index += 1;
        let track_ptr = unsafe {
          Sequence::fns().getTrackAtIndex.unwrap()(self.sequence.as_mut().cptr_mut(), index)
        };
        if !track_ptr.is_null() {
          self.count_left -= 1;
          return Some(SequenceTrackMut::new(
            track_ptr,
            index,
            self.sequence.as_ptr(),
            unsafe { self.sequence.as_mut().track_instrument_mut(index) },
          ));
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.count_total, Some(self.count_total))
  }
}
impl ExactSizeIterator for SequenceTrackIterMut<'_> {}
impl core::iter::FusedIterator for SequenceTrackIterMut<'_> {}
