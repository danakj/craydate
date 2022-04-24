//! This module re-exports playdate_sys types with more consistent names.

pub use playdate_sys::playdate_display as CDisplayApi;
pub use playdate_sys::playdate_file as CFileApi;
pub use playdate_sys::playdate_graphics as CGraphicsApi;
pub use playdate_sys::playdate_sound as CSoundApi;
pub use playdate_sys::playdate_sound_channel as CSoundChannelApi;
pub use playdate_sys::playdate_sound_effect as CSoundEffectApi;
pub use playdate_sys::playdate_sound_effect_bitcrusher as CSoundEffectBitCrusherApi;
pub use playdate_sys::playdate_sound_effect_delayline as CSoundEffectDelayLineApi;
pub use playdate_sys::playdate_sound_effect_onepolefilter as CSoundEffectOnePoleFilterApi;
pub use playdate_sys::playdate_sound_effect_overdrive as CSoundEffectOverDriveApi;
pub use playdate_sys::playdate_sound_effect_ringmodulator as CSoundEffectRingModulatorApi;
pub use playdate_sys::playdate_sound_effect_twopolefilter as CSoundEffectTwoPoleFilterApi;
pub use playdate_sys::playdate_sound_envelope as CSoundEnvelopeApi;
pub use playdate_sys::playdate_sound_fileplayer as CSoundFilePlayerApi;
pub use playdate_sys::playdate_sound_instrument as CSoundInstrumentApi;
pub use playdate_sys::playdate_sound_lfo as CSoundLfoApi;
pub use playdate_sys::playdate_sound_sample as CSoundSampleApi;
pub use playdate_sys::playdate_sound_sampleplayer as CSoundSamplePlayerApi;
pub use playdate_sys::playdate_sound_sequence as CSoundSequenceApi;
pub use playdate_sys::playdate_sound_source as CSoundSourceApi;
pub use playdate_sys::playdate_sound_synth as CSoundSynthApi;
pub use playdate_sys::playdate_sound_track as CSoundTrackApi;
pub use playdate_sys::playdate_sys as CSystemApi;
pub use playdate_sys::playdate_video as CVideoApi;
pub use playdate_sys::AudioSample as CAudioSample;
pub use playdate_sys::BitCrusher as CBitCrusher;
pub use playdate_sys::ControlSignal as CControlSignal;
pub use playdate_sys::DelayLine as CDelayLine;
pub use playdate_sys::DelayLineTap as CDelayLineTap;
pub use playdate_sys::FilePlayer as CFilePlayer;
pub use playdate_sys::FileStat as CFileStat;
pub use playdate_sys::LCDBitmap as CBitmap;
pub use playdate_sys::LCDColor as CLCDColor;
pub use playdate_sys::LCDFont as CFont;
pub use playdate_sys::LCDFontGlyph as CFontGlyph;
pub use playdate_sys::LCDFontPage as CFontPage;
pub use playdate_sys::LCDPattern as CLCDPattern;
pub use playdate_sys::LCDRect as CLCDRect;
pub use playdate_sys::LCDVideoPlayer as CVideoPlayer;
pub use playdate_sys::LFOType as CSynthLfoType;
pub use playdate_sys::OnePoleFilter as COnePoleFilter;
pub use playdate_sys::Overdrive as COverdrive;
pub use playdate_sys::PDButtons as CButtons;
pub use playdate_sys::PDMenuItem as CMenuItem;
pub use playdate_sys::PDStringEncoding as CStringEncoding;
pub use playdate_sys::PDSynth as CSynth;
pub use playdate_sys::PDSynthEnvelope as CSynthEnvelope;
pub use playdate_sys::PDSynthInstrument as CSynthInstrument;
pub use playdate_sys::PDSynthLFO as CSynthLfo;
pub use playdate_sys::PDSynthSignalValue as CSynthSignalValue;
pub use playdate_sys::PDSystemEvent as CSystemEvent;
pub use playdate_sys::PlaydateAPI as CPlaydateApi;
pub use playdate_sys::RingModulator as CRingModulator;
pub use playdate_sys::SDFile as COpenFile;
pub use playdate_sys::SamplePlayer as CSamplePlayer;
pub use playdate_sys::SequenceTrack as CSequenceTrack;
pub use playdate_sys::SoundChannel as CSoundChannel;
pub use playdate_sys::SoundEffect as CSoundEffect;
pub use playdate_sys::SoundFormat as CSoundFormat;
pub use playdate_sys::SoundSequence as CSoundSequence;
pub use playdate_sys::SoundSource as CSoundSource;
pub use playdate_sys::SoundWaveform as CSoundWaveform;
pub use playdate_sys::TwoPoleFilter as CTwoPoleFilter;

pub use crate::ctypes_enums::*;

/// CButtons come in groups of 3, so this is a convenience grouping of them.
#[derive(Debug, Copy, Clone)]
pub struct PDButtonsSet {
  pub current: CButtons,
  pub pushed: CButtons,
  pub released: CButtons,
}
