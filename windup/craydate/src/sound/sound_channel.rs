use alloc::rc::Rc;
use core::ptr::NonNull;

use super::effects::sound_effect::SoundEffect;
use super::signals::synth_signal::{SynthSignal, SynthSignalSubclass};
use super::sources::sound_source::SoundSource;
use super::volume::Volume;
use super::Sound;
use crate::capi_state::CApiState;
use crate::clamped_float::ClampedFloatInclusive;
use crate::ctypes::*;
use crate::error::Error;

/// A channel is where sound is played to, once it has been added to the system via
/// `Sound::add_channel()`. Sounds can be played into a `SoundChannel` by attaching a `SoundSource`
/// with `add_source()`.
#[derive(Debug)]
pub struct SoundChannel {
  // This class holds an Rc but is not Clone. This allows it to know when the Rc is going away, in
  // order to clean up other related stuff.
  ptr: Rc<NonNull<CSoundChannel>>,
  // True if owned by the application, false if owned by Playdate.
  owned: bool,
  // Tracks if the `SoundChannel` has be "added" to the sound system of the device with
  // `Sound::add_channel()`.
  added: bool,
  volume_modulator: Option<SynthSignal>,
  pan_modulator: Option<SynthSignal>,
  dry_level_signal: SynthSignal,
  wet_level_signal: SynthSignal,
}
impl SoundChannel {
  fn from_ptr(ptr: *mut CSoundChannel, owned: bool) -> SoundChannel {
    let dry_level_signal = SynthSignal::new(
      unsafe { Self::fns().getDryLevelSignal.unwrap()(ptr) },
      Rc::new(LevelSignal {}),
    );
    let wet_level_signal = SynthSignal::new(
      unsafe { Self::fns().getWetLevelSignal.unwrap()(ptr) },
      Rc::new(LevelSignal {}),
    );
    SoundChannel {
      ptr: Rc::new(NonNull::new(ptr).unwrap()),
      owned,
      added: false,
      volume_modulator: None,
      pan_modulator: None,
      dry_level_signal,
      wet_level_signal,
    }
  }

  pub fn new() -> SoundChannel {
    Self::from_ptr(unsafe { Self::fns().newChannel.unwrap()() }, true)
  }

  pub(crate) fn new_system_channel(ptr: *mut CSoundChannel) -> SoundChannel {
    Self::from_ptr(ptr, false)
  }
  pub(crate) fn is_system_channel(&self) -> bool {
    !self.owned
  }

  pub(crate) fn set_added(&mut self, added: bool) {
    assert!(self.owned);
    self.added = added
  }

  /// Adds the `source` to this channel, so it plays into the channel.
  ///
  /// # Return
  /// Returns `Error::AlreadyAttachedError` if the `source` is already attached to a channel or (for
  /// a Synth) to an Instrument.
  pub fn add_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) -> Result<(), Error> {
    source.as_mut().attach_to_channel(&self.ptr)
  }
  /// Remove the `source` from this channel.
  ///
  /// # Return
  /// Returns `Error::NotFoundError` if the `source` is not attached to the the channel.
  pub fn remove_source<T: AsMut<SoundSource>>(&mut self, source: &mut T) -> Result<(), Error> {
    source.as_mut().detach_from_channel(&self.ptr)
  }

  /// Attach the `sound_effect` to this channel, so it plays into the channel.
  ///
  /// # Return
  /// Returns `Error::AlreadyAttachedError` if the `source` is already attached to a channel or (for
  /// a Synth) to an Instrument.
  pub fn add_sound_effect<T: AsMut<SoundEffect>>(
    &mut self,
    sound_effect: &mut T,
  ) -> Result<(), Error> {
    sound_effect.as_mut().attach_to_channel(&self.ptr)
  }
  /// Remove the `sound_effect` from this channel.
  ///
  /// # Return
  /// Returns `Error::NotFoundError` if the `source` is not attached to the the channel.
  pub fn remove_sound_effect<T: AsMut<SoundEffect>>(
    &mut self,
    sound_effect: &mut T,
  ) -> Result<(), Error> {
    sound_effect.as_mut().detach_from_channel(&self.ptr)
  }

  /// Gets the volume for the channel, in the range [0-1].
  pub fn volume(&self) -> Volume {
    // getVolume() takes a mutable pointer but doesn't mutate any visible state.
    unsafe { Self::fns().getVolume.unwrap()(self.cptr() as *mut _).into() }
  }
  /// Sets the volume for the channel, in the range [0-1].
  pub fn set_volume(&mut self, volume: Volume) {
    unsafe { Self::fns().setVolume.unwrap()(self.cptr_mut(), volume.into()) }
  }
  /// Sets a signal to modulate the channel volume.
  pub fn set_volume_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal|
      // setVolumeModulator() takes a mutable pointer to the modulator but there is no visible state
      // on the modulator.
      signal.as_ref().cptr() as *mut _);
    unsafe { Self::fns().setVolumeModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.volume_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the channel volume.
  pub fn volume_modulator(&mut self) -> Option<&SynthSignal> {
    self.volume_modulator.as_ref()
  }

  /// Sets the pan parameter for the channel.
  ///
  /// The pan value is between -1 which is left and 1 which is right. 0 is center.
  pub fn set_pan(&mut self, pan: ClampedFloatInclusive<-1, 1>) {
    unsafe { Self::fns().setPan.unwrap()(self.cptr_mut(), pan.into()) }
  }
  /// Sets a signal to modulate the channel pan.
  pub fn set_pan_modulator<T: AsRef<SynthSignal>>(&mut self, signal: Option<&T>) {
    let modulator_ptr = signal.map_or_else(core::ptr::null_mut, |signal|
      // setPanModulator() takes a mutable pointer to the modulator but there is no visible state on
      // the modulator.
      signal.as_ref().cptr() as *mut _);
    unsafe { Self::fns().setPanModulator.unwrap()(self.cptr_mut(), modulator_ptr) }
    self.pan_modulator = signal.map(|signal| signal.as_ref().clone());
  }
  /// Gets the current signal modulating the channel pan.
  pub fn pan_modulator(&mut self) -> Option<&SynthSignal> {
    self.pan_modulator.as_ref()
  }

  /// Returns a signal that follows the volume of the channel before effects are applied.
  pub fn dry_level_signal(&mut self) -> &SynthSignal {
    &self.dry_level_signal
  }
  /// Returns a signal that follows the volume of the channel after effects are applied.
  pub fn wet_level_signal(&mut self) -> &SynthSignal {
    &self.wet_level_signal
  }

  pub(crate) fn cptr(&self) -> *const CSoundChannel {
    self.ptr.as_ptr()
  }
  pub(crate) fn cptr_mut(&mut self) -> *mut CSoundChannel {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_sound_channel {
    unsafe { &*CApiState::get().csound.channel }
  }
}

impl Drop for SoundChannel {
  fn drop(&mut self) {
    if self.added {
      unsafe { Sound::fns().removeChannel.unwrap()(self.cptr_mut()) }
    }
    if self.owned {
      unsafe { Self::fns().freeChannel.unwrap()(self.cptr_mut()) }
    }
  }
}

/// A LevelSignal is for a SynthSignal that is owned by playdate, so there's nothing to own in the
/// SynthSignalSubclass.
struct LevelSignal {}
impl SynthSignalSubclass for LevelSignal {}
