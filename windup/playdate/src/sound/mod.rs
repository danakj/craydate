pub(crate) mod audio_sample;
pub(crate) mod effects;
pub(crate) mod loop_sound_span;
pub(crate) mod midi;
pub(crate) mod signals;
pub(crate) mod sound_channel;
pub(crate) mod sound_format;
pub(crate) mod sources;
pub(crate) mod stereo_volume;

pub use audio_sample::AudioSample;
pub use effects::bit_crusher::BitCrusher;
pub use effects::delay_line::DelayLine;
pub use effects::one_pole_filter::OnePoleFilter;
pub use effects::overdrive::Overdrive;
pub use effects::ring_modulator::RingModulator;
pub use effects::sound_effect::SoundEffect;
pub use effects::two_pole_filter::TwoPoleFilter;
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
pub use sources::delay_line_tap::DelayLineTap;
pub use sources::callback_source::CallbackSource;
pub use sources::file_player::FilePlayer;
pub use sources::instrument::{Instrument, VoiceId};
pub use sources::sample_player::SamplePlayer;
pub use sources::sound_source::{AsSoundSource, SoundSource};
pub use sources::synth::{Synth, SynthGenerator, SynthGeneratorVTable, SynthRender};
pub use stereo_volume::StereoVolume;

use crate::callbacks::AllowNull;
use crate::capi_state::CApiState;
use crate::time::TimeTicks;

pub(crate) const SAMPLE_FRAMES_PER_SEC: i32 = 44_100;

/// A callback builder for a closure to be called on sound completion events.
pub type SoundCompletionCallback<'a, T, F, S> =
  crate::callbacks::CallbackBuilder<'a, T, F, AllowNull, S>;

/// Access to the speaker and headphone outputs of the Playdate device, along with the audio clock.
#[derive(Debug)]
pub struct Sound {
  default_channel: SoundChannel,
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

  // TODO: getHeadphoneState

  // BUG: Microphone monitoring functions are missing:
  // https://devforum.play.date/t/c-api-missing-microphone-monitoring-functions/4926

  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound {
    CApiState::get().csound
  }
}
