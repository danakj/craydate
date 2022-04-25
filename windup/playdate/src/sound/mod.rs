pub(crate) mod audio_sample;
pub(crate) mod effects;
pub(crate) mod headphone_state;
pub(crate) mod loop_sound_span;
pub(crate) mod midi;
pub(crate) mod signals;
pub(crate) mod sound_channel;
pub(crate) mod sound_format;
pub(crate) mod sources;
pub(crate) mod volume;

pub use audio_sample::AudioSample;
pub use effects::bit_crusher::BitCrusher;
pub use effects::delay_line::DelayLine;
pub use effects::one_pole_filter::OnePoleFilter;
pub use effects::overdrive::Overdrive;
pub use effects::ring_modulator::RingModulator;
pub use effects::sound_effect::SoundEffect;
pub use effects::two_pole_filter::TwoPoleFilter;
pub use headphone_state::HeadphoneState;
pub use loop_sound_span::LoopTimeSpan;
pub use midi::midi_note_range::MidiNoteRange;
pub use midi::sequence::Sequence;
pub use midi::sequence_track::SequenceTrack;
pub use midi::track_note::TrackNote;
pub use signals::control::Control;
pub use signals::envelope::Envelope;
pub use signals::lfo::Lfo;
pub use signals::synth_signal::{AsSynthSignal, SynthSignal};
pub use sound_channel::SoundChannel;
pub use sound_format::*;
pub use sources::callback_source::CallbackSource;
pub use sources::delay_line_tap::DelayLineTap;
pub use sources::file_player::FilePlayer;
pub use sources::instrument::{Instrument, VoiceId};
pub use sources::sample_player::SamplePlayer;
pub use sources::sound_source::{AsSoundSource, SoundSource};
pub use sources::synth::{Synth, SynthGenerator, SynthGeneratorVTable, SynthRender};
pub use volume::{StereoVolume, Volume};

use crate::callback_builder::{AllowNull, CallbackBuilder, CallbackBuilderWithArg, Constructed};
use crate::capi_state::CApiState;
use crate::time::TimeTicks;

pub(crate) const SAMPLE_FRAMES_PER_SEC: i32 = 44_100;

/// A callback builder for a closure to be called on sound completion events.
pub type SoundCompletionCallback<'a, T, F, S> = CallbackBuilder<'a, T, F, AllowNull, S>;

/// A callback builder for a closure to be called on headphone change events.
pub type HeadphoneChangeCallback<'a, T, F, S> =
  CallbackBuilderWithArg<'a, HeadphoneState, T, F, AllowNull, S>;

/// Access to the speaker and headphone outputs of the Playdate device, along with the audio clock.
#[derive(Debug)]
pub struct Sound {
  default_channel: SoundChannel, // TODO: Move to CApiState
}
impl Sound {
  pub(crate) fn new() -> Self {
    Sound {
      default_channel: SoundChannel::new_system_channel(unsafe {
        Self::fns().getDefaultChannel.unwrap()()
      }),
    }
  }

  /// The default `SoundChannel`. Attaching a `SoundSource` to it will play from the device.
  pub fn default_channel(&self) -> &SoundChannel {
    &self.default_channel
  }
  /// The default `SoundChannel`. Attaching a `SoundSource` to it will play from the device.
  pub fn default_channel_mut(&mut self) -> &mut SoundChannel {
    &mut self.default_channel
  }

  /// Add a user-created `SoundChannel` to have it play from the device.
  pub fn add_channel(&mut self, channel: &mut SoundChannel) {
    if !channel.is_system_channel() {
      channel.set_added(true);
      unsafe { Self::fns().addChannel.unwrap()(channel.cptr_mut()) };
    }
  }
  /// Remove a user-created `SoundChannel` to no longer have it play from the device.
  ///
  /// Does nothing if the `SoundChannel` was not already added with `add_channel()`.
  pub fn remove_channel(&mut self, channel: &mut SoundChannel) {
    if !channel.is_system_channel() {
      channel.set_added(false);
      unsafe { Self::fns().removeChannel.unwrap()(channel.cptr_mut()) }
    }
  }

  /// Returns the sound engine’s current time value.
  pub fn current_sound_time(&self) -> TimeTicks {
    TimeTicks::from_sample_frames(unsafe { Self::fns().getCurrentTime.unwrap()() })
  }

  /// Force audio output to the given outputs, regardless of headphone status.
  pub fn set_active_outputs(&self, headphone: bool, speaker: bool) {
    unsafe { Self::fns().setOutputsActive.unwrap()(headphone as i32, speaker as i32) };
  }

  // TODO: setMicCallback - consider recordToSample() instead like for LUA:
  // https://sdk.play.date/1.10.0/Inside%20Playdate.html#f-sound.micinput.recordToSample

  /// Sets a callback to be called when the headphone state changes.
  ///
  /// When a callback is set, then audio will _not_ automatically switch to the headphones when they
  /// are plugged in or removed. Instead, the application will need to call
  /// `Sound::set_active_outputs()` to move sound to and from the headphones.
  ///
  /// The callback will be registered as a system event, and the application will be notified to run
  /// the callback via a `SystemEvent::Callback` event. When that occurs, the application's
  /// `Callbacks` object which was used to construct the `change_callback` can be `run()` to
  /// execute the closure bound in the `change_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// sound.set_headphone_change_callback(HeadphoneChangeCallback::with(&mut callbacks).call(
  ///   |state: HeadphoneState, i: i32| {
  ///     println("headphone changed");
  ///   })
  /// );
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.run(12);
  ///   }
  /// }
  /// ```
  pub fn set_headphone_change_callback<'a, T, F: Fn(HeadphoneState, T) + 'static>(
    &mut self,
    change_callback: HeadphoneChangeCallback<'a, T, F, Constructed>,
  ) {
    let mut headphone_callback = CApiState::get().headphone_change_callback.borrow_mut();
    *headphone_callback = None;

    let func = change_callback.into_inner().and_then(|(callbacks, cb)| {
      let (func, reg) = callbacks.add_headphone_change(cb);
      *headphone_callback = Some(reg);
      Some(func)
    });
    let mut headphone = 0;
    let mut mic = 0;
    unsafe { Self::fns().getHeadphoneState.unwrap()(&mut headphone, &mut mic, func) }

    // Save the function pointer so we can call getHeadphoneState() without changing it elsewhere.
    *CApiState::get().headphone_change_func.borrow_mut() = func;
  }

  /// Returns the current headphones state, which includes if they are plugged in and if they have a
  /// microphone.
  pub fn headphone_state(&self) -> HeadphoneState {
    // Grab the function pointer last passed to getHeadphoneState() in
    // `set_headphone_change_callback()`, so that we don't change that here.
    let func = CApiState::get().headphone_change_func.borrow().clone();

    let mut headphone = 0;
    let mut mic = 0;
    unsafe { Self::fns().getHeadphoneState.unwrap()(&mut headphone, &mut mic, func) };
    HeadphoneState::new(headphone != 0, mic != 0)
  }

  // BUG: Microphone monitoring functions are missing:
  // https://devforum.play.date/t/c-api-missing-microphone-monitoring-functions/4926

  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound {
    CApiState::get().csound
  }
}
