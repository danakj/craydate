use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::sources::instrument::InstrumentRef;
use super::super::SoundCompletionCallback;
use super::sequence_track::{SequenceTrack, SequenceTrackRef, UnownedSequenceTrack,
                            UnownedSequenceTrackMut};
use crate::callbacks::{Constructed, RegisteredCallback};
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;

// TODO: If we want to be able to add and remove tracks at will, when we probably need two kinds of
// sequences, where one is user-defined and owned tracks, and one is loaded from a MIDI file and is
// all unowned tracks. Then we only have to track ownership of the things the user created, and we
// don't have to worry about unowned tracks being added to a Sequence, which would get weird
// otherwise.

/// Build a Sequence from a set of SequenceTracks.
pub struct SequenceBuilder<'a> {
  tracks: Vec<(Option<u32>, &'a SequenceTrackRef)>,
}
impl<'a> SequenceBuilder<'a> {
  pub fn new() -> Self {
    SequenceBuilder { tracks: Vec::new() }
  }

  /// Add a track at a given index.
  ///
  /// If another track was already specified at the same index, it would be replaced.
  pub fn add_track_at_index(mut self, index: u32, track: &'a SequenceTrackRef) -> Self {
    self.tracks.push((Some(index), track));
    self
  }

  /// Consume the `SequenceBuilder` and construct the `Sequence`.
  pub fn build(self) -> Sequence<'a> {
    let seq = Sequence::new();
    for (index, track) in self.tracks {
      match index {
        Some(index) => unsafe {
          Sequence::fns().setTrackAtIndex.unwrap()(seq.cptr(), track.cptr(), index)
        },
        None => {
          // TODO: When we have addTrack() available. But it's missing a parameter:
          // https://devforum.play.date/t/c-api-soundsequence-addtrack-missing-parameter/4509
          unreachable!()
        }
      }
    }
    seq
  }
}

/// Represents a MIDI music file, as a collection of `SequenceTrack`s that can be played together.
pub struct Sequence<'a> {
  ptr: NonNull<CSoundSequence>,
  finished_callback: Option<RegisteredCallback>,
  _marker: PhantomData<&'a SequenceTrackRef>,
}
impl Sequence<'_> {
  fn from_ptr(ptr: *mut CSoundSequence) -> Self {
    Sequence {
      ptr: NonNull::new(ptr).unwrap(),
      finished_callback: None,
      _marker: PhantomData,
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
  ///
  /// To load a midi file and connect it to the sound system, there are a few steps:
  /// 1. Load the MIDI file with this function.
  /// 2. Create an `Instrument` and set it as the `Instrument` for each `SequenceTrack` in the
  ///    `Sequence`.
  /// 3. Attach each `Instrument` to a `SoundChannel` to hear the MIDI play there, such as the
  ///    `SoundChannel` returned from `Sound::default_channel_mut()`.
  /// 3. Create one or more (up to the sum of `SequenceTrack::polyphony()` for all `SequenceTrack`s
  ///   many) `Synth` objects, with a `SoundWaveform`. Set the various parameters to taste.
  /// 4. Attach the `Synth` objects to the `Instruments`. 
  /// 5. And now you can `play()` the `Sequence`.
  /// 
  /// # Example
  /// ```
  /// let mut synths = Vec::new();
  /// let mut instruments = Vec::new();
  /// let mut sequence = Sequence::from_midi_file("song.mid").unwrap();
  /// for mut track in sequence.tracks_mut() {
  ///   let mut instrument = Instrument::new();
  ///   instrument.set_volume(StereoVolume { left: 0.3, right: 0.3 });
  ///   api.sound.default_channel_mut().attach_source(&mut instrument).unwrap();
  ///   track.set_instrument(&mut instrument);
  /// 
  ///   for _ in 0..track.polyphony() {
  ///     let mut synth = Synth::new_with_waveform(SoundWaveform::kWaveformSquare);
  ///     synth.set_attack_time(TimeDelta::from_milliseconds(0));
  ///     synth.set_decay_time(TimeDelta::from_milliseconds(200));
  ///     synth.set_sustain_level(0.3);
  ///     synth.set_release_time(TimeDelta::from_milliseconds(500));
  ///     let instrument = track.instrument_mut().unwrap();
  ///     instrument.add_voice(&mut synth, MidiNoteRange::All, 0.0).unwrap();
  ///
  ///     synths.push(synth);
  ///   }
  ///   instruments.push(instrument);
  /// }
  /// sequence.play(SoundCompletionCallback::none());
  /// ```
  pub fn from_midi_file(path: &str) -> Result<Self, Error> {
    let seq = Self::new();
    let r = unsafe {
      Self::fns().loadMidiFile.unwrap()(seq.cptr(), path.to_null_terminated_utf8().as_ptr())
    };
    match r {
      0 => Err(Error::LoadMidiFileError),
      _ => Ok(seq),
    }
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
  pub fn tracks<'a>(&'a self) -> impl Iterator<Item = UnownedSequenceTrack> + 'a {
    SequenceTrackIter {
      seq: self,
      next: 0,
      count: self.tracks_count(),
    }
  }
  /// Returns a mutable iterator over all the tracks in the `Sequence`.
  pub fn tracks_mut<'a>(&'a mut self) -> impl Iterator<Item = UnownedSequenceTrackMut> + 'a {
    SequenceTrackIterMut {
      seq: self,
      next: 0,
      count: self.tracks_count(),
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

impl Drop for Sequence<'_> {
  fn drop(&mut self) {
    unsafe { Self::fns().freeSequence.unwrap()(self.cptr()) }
  }
}

struct SequenceTrackIter<'a, 'tracks> {
  seq: &'a Sequence<'tracks>,
  next: u32,
  count: u32,
}
impl<'a> Iterator for SequenceTrackIter<'a, '_> {
  type Item = UnownedSequenceTrack<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.next == self.count {
      None
    } else {
      let i = self.next;
      self.next += 1;
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.seq.cptr(), i) };
      let inst_ptr = unsafe { SequenceTrack::fns().getInstrument.unwrap()(track_ptr) };
      let inst_ref = if inst_ptr.is_null() {
        None
      } else {
        Some(InstrumentRef::from_ptr(inst_ptr))
      };
      Some(UnownedSequenceTrack::new(track_ptr, inst_ref))
    }
  }
}

struct SequenceTrackIterMut<'a, 'tracks> {
  seq: &'a Sequence<'tracks>,
  next: u32,
  count: u32,
}
impl<'a> Iterator for SequenceTrackIterMut<'a, '_> {
  type Item = UnownedSequenceTrackMut<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.next == self.count {
      None
    } else {
      let i = self.next;
      self.next += 1;
      let track_ptr = unsafe { Sequence::fns().getTrackAtIndex.unwrap()(self.seq.cptr(), i) };
      let inst_ptr = unsafe { SequenceTrack::fns().getInstrument.unwrap()(track_ptr) };
      let inst_ref = if inst_ptr.is_null() {
        None
      } else {
        Some(InstrumentRef::from_ptr(inst_ptr))
      };
      Some(UnownedSequenceTrackMut::new(track_ptr, inst_ref))
    }
  }
}
