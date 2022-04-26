//! This module re-exports playdate_sys types with more consistent names.

pub use craydate_sys::playdate_display as CDisplayApi;
pub use craydate_sys::playdate_file as CFileApi;
pub use craydate_sys::playdate_graphics as CGraphicsApi;
pub use craydate_sys::playdate_sound as CSoundApi;
pub use craydate_sys::playdate_sound_channel as CSoundChannelApi;
pub use craydate_sys::playdate_sound_effect as CSoundEffectApi;
pub use craydate_sys::playdate_sound_effect_bitcrusher as CSoundEffectBitCrusherApi;
pub use craydate_sys::playdate_sound_effect_delayline as CSoundEffectDelayLineApi;
pub use craydate_sys::playdate_sound_effect_onepolefilter as CSoundEffectOnePoleFilterApi;
pub use craydate_sys::playdate_sound_effect_overdrive as CSoundEffectOverDriveApi;
pub use craydate_sys::playdate_sound_effect_ringmodulator as CSoundEffectRingModulatorApi;
pub use craydate_sys::playdate_sound_effect_twopolefilter as CSoundEffectTwoPoleFilterApi;
pub use craydate_sys::playdate_sound_envelope as CSoundEnvelopeApi;
pub use craydate_sys::playdate_sound_fileplayer as CSoundFilePlayerApi;
pub use craydate_sys::playdate_sound_instrument as CSoundInstrumentApi;
pub use craydate_sys::playdate_sound_lfo as CSoundLfoApi;
pub use craydate_sys::playdate_sound_sample as CSoundSampleApi;
pub use craydate_sys::playdate_sound_sampleplayer as CSoundSamplePlayerApi;
pub use craydate_sys::playdate_sound_sequence as CSoundSequenceApi;
pub use craydate_sys::playdate_sound_source as CSoundSourceApi;
pub use craydate_sys::playdate_sound_synth as CSoundSynthApi;
pub use craydate_sys::playdate_sound_track as CSoundTrackApi;
pub use craydate_sys::playdate_sys as CSystemApi;
pub use craydate_sys::playdate_video as CVideoApi;
pub use craydate_sys::AudioSample as CAudioSample;
pub use craydate_sys::BitCrusher as CBitCrusher;
pub use craydate_sys::ControlSignal as CControlSignal;
pub use craydate_sys::DelayLine as CDelayLine;
pub use craydate_sys::DelayLineTap as CDelayLineTap;
pub use craydate_sys::FilePlayer as CFilePlayer;
pub use craydate_sys::FileStat as CFileStat;
pub use craydate_sys::LCDBitmap as CBitmap;
pub use craydate_sys::LCDColor as CLCDColor;
pub use craydate_sys::LCDFont as CFont;
pub use craydate_sys::LCDFontGlyph as CFontGlyph;
pub use craydate_sys::LCDFontPage as CFontPage;
pub use craydate_sys::LCDPattern as CLCDPattern;
pub use craydate_sys::LCDRect as CLCDRect;
pub use craydate_sys::LCDVideoPlayer as CVideoPlayer;
pub use craydate_sys::LFOType as CSynthLfoType;
pub use craydate_sys::OnePoleFilter as COnePoleFilter;
pub use craydate_sys::Overdrive as COverdrive;
pub use craydate_sys::PDButtons as CButtons;
pub use craydate_sys::PDMenuItem as CMenuItem;
pub use craydate_sys::PDStringEncoding as CStringEncoding;
pub use craydate_sys::PDSynth as CSynth;
pub use craydate_sys::PDSynthEnvelope as CSynthEnvelope;
pub use craydate_sys::PDSynthInstrument as CSynthInstrument;
pub use craydate_sys::PDSynthLFO as CSynthLfo;
pub use craydate_sys::PDSynthSignalValue as CSynthSignalValue;
pub use craydate_sys::PDSystemEvent as CSystemEvent;
pub use craydate_sys::PlaydateAPI as CPlaydateApi;
pub use craydate_sys::RingModulator as CRingModulator;
pub use craydate_sys::SDFile as COpenFile;
pub use craydate_sys::SamplePlayer as CSamplePlayer;
pub use craydate_sys::SequenceTrack as CSequenceTrack;
pub use craydate_sys::SoundChannel as CSoundChannel;
pub use craydate_sys::SoundEffect as CSoundEffect;
pub use craydate_sys::SoundFormat as CSoundFormat;
pub use craydate_sys::SoundSequence as CSoundSequence;
pub use craydate_sys::SoundSource as CSoundSource;
pub use craydate_sys::SoundWaveform as CSoundWaveform;
pub use craydate_sys::TwoPoleFilter as CTwoPoleFilter;

pub use crate::ctypes_enums::*;

/// CButtons come in groups of 3, so this is a convenience grouping of them.
#[derive(Debug, Copy, Clone)]
pub struct PDButtonsSet {
  pub current: CButtons,
  pub pushed: CButtons,
  pub released: CButtons,
}
