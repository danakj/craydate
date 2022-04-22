pub(crate) mod audio_sample;
pub(crate) mod effects;
pub(crate) mod loop_sound_span;
pub(crate) mod midi;
pub(crate) mod sample_frames;
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
pub use sample_frames::SampleFrames;
pub use signals::control::Control;
pub use signals::envelope::Envelope;
pub use signals::lfo::Lfo;
pub use signals::synth_signal::SynthSignal;
pub use sound_channel::{SoundChannel, SoundChannelRef};
pub use sound_format::*;
pub use sources::delay_line_tap::DelayLineTap;
pub use sources::file_player::FilePlayer;
pub use sources::instrument::{Instrument, VoiceId};
pub use sources::sample_player::SamplePlayer;
pub use sources::sound_source::SoundSource;
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
  default_channel: SoundChannelRef,
}
impl Sound {
  pub(crate) fn new() -> Self {
    Sound {
      default_channel: SoundChannelRef::from_ptr(unsafe {
        Self::fns().getDefaultChannel.unwrap()()
      }),
    }
  }

  pub fn default_channel(&self) -> &SoundChannelRef {
    &self.default_channel
  }
  pub fn default_channel_mut(&mut self) -> &mut SoundChannelRef {
    &mut self.default_channel
  }

  pub fn add_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(true);
    unsafe { Self::fns().addChannel.unwrap()(channel.cptr()) };
  }
  pub fn remove_channel(&mut self, channel: &mut SoundChannel) {
    channel.set_added(false);
    unsafe { Self::fns().removeChannel.unwrap()(channel.cptr()) }
  }

  /// Returns the sound engine’s current time value.
  pub fn current_sound_time(&self) -> TimeTicks {
    let frames = self.current_sound_time_frames();
    TimeTicks::from_sample_frames(frames.0)
  }

  /// Returns the sound engine’s current time value, in units of sample frames, 44,100 per second.
  pub fn current_sound_time_frames(&self) -> SampleFrames {
    SampleFrames(unsafe { Self::fns().getCurrentTime.unwrap()() })
  }

  /// Force audio output to the given outputs, regardless of headphone status.
  pub fn set_active_outputs(&self, headphone: bool, speaker: bool) {
    unsafe { Self::fns().setOutputsActive.unwrap()(headphone as i32, speaker as i32) };
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_sound {
    CApiState::get().csound
  }
}
