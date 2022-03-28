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
pub use playdate_sys::FileStat as CFileStat;
pub use playdate_sys::LCDBitmap as CLCDBitmap;
pub use playdate_sys::LCDColor as CLCDColor;
pub use playdate_sys::LCDFont as CLCDFont;
pub use playdate_sys::LCDFontGlyph as CLCDFontGlyph;
pub use playdate_sys::LCDFontPage as CLCDFontPage;
pub use playdate_sys::LCDPattern as CLCDPattern;
pub use playdate_sys::LCDRect as CLCDRect;
pub use playdate_sys::LCDVideoPlayer as CVideoPlayer;
pub use playdate_sys::PDButtons as CButtons;
pub use playdate_sys::PDSystemEvent as CSystemEvent;
pub use playdate_sys::PlaydateAPI as CPlaydateApi;
pub use playdate_sys::SDFile as COpenFile;
pub use playdate_sys::SoundChannel as CSoundChannel;
pub use playdate_sys::SoundEffect as CSoundEffect;
pub use playdate_sys::SoundFormat as CSoundFormat;
pub use playdate_sys::SoundSequence as CSoundSequence;
pub use playdate_sys::SoundSource as CSoundSource;
pub use playdate_sys::SoundWaveform as CSoundWaveform;

pub use crate::ctypes_enums::*;

/// CButtons come in groups of 3, so this is a convenience grouping of them.
#[derive(Debug, Copy, Clone)]
pub struct PDButtonsSet {
  pub current: CButtons,
  pub pushed: CButtons,
  pub released: CButtons,
}
