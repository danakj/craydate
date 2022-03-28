//! This module re-exports playdate_sys types with more consistent names.
pub use playdate_sys::playdate_display as CDisplay;
pub use playdate_sys::playdate_file as CFile;
pub use playdate_sys::playdate_graphics as CGraphics;
pub use playdate_sys::playdate_sound as CSound;
pub use playdate_sys::playdate_sound_channel as CSoundChannel;
pub use playdate_sys::playdate_sound_effect as CSoundEffect;
pub use playdate_sys::playdate_sound_effect_bitcrusher as CSoundEffectBitCrusher;
pub use playdate_sys::playdate_sound_effect_delayline as CSoundEffectDelayLine;
pub use playdate_sys::playdate_sound_effect_onepolefilter as CSoundEffectOnePoleFilter;
pub use playdate_sys::playdate_sound_effect_overdrive as CSoundEffectOverDrive;
pub use playdate_sys::playdate_sound_effect_ringmodulator as CSoundEffectRingModulator;
pub use playdate_sys::playdate_sound_effect_twopolefilter as CSoundEffectTwoPoleFilter;
pub use playdate_sys::playdate_sound_envelope as CSoundEnvelope;
pub use playdate_sys::playdate_sound_fileplayer as CSoundFilePlayer;
pub use playdate_sys::playdate_sound_instrument as CSoundInstrument;
pub use playdate_sys::playdate_sound_lfo as CSoundLfo;
pub use playdate_sys::playdate_sound_sample as CSoundSample;
pub use playdate_sys::playdate_sound_sampleplayer as CSoundSamplePlayer;
pub use playdate_sys::playdate_sound_sequence as CSoundSequence;
pub use playdate_sys::playdate_sound_source as CSoundSource;
pub use playdate_sys::playdate_sound_synth as CSoundSynth;
pub use playdate_sys::playdate_sound_track as CSoundTrack;
pub use playdate_sys::playdate_sys as CSystem;
pub use playdate_sys::playdate_video as CVideo;
pub use playdate_sys::FileStat as CFileStat;
pub use playdate_sys::LCDBitmap as CLCDBitmap;
pub use playdate_sys::LCDColor as CLCDColor;
pub use playdate_sys::LCDFont as CLCDFont;
pub use playdate_sys::LCDFontGlyph as CLCDFontGlyph;
pub use playdate_sys::LCDFontPage as CLCDFontPage;
pub use playdate_sys::LCDPattern as CLCDPattern;
pub use playdate_sys::LCDRect as CLCDRect;
pub use playdate_sys::LCDVideoPlayer as CVideoPlayer;
pub use playdate_sys::PDButtons;
// Bitflags.
pub use playdate_sys::PDPeripherals;
pub use playdate_sys::PDSystemEvent as CSystemEvent;
pub use playdate_sys::PlaydateAPI as CApi;
pub use playdate_sys::SDFile as COpenFile;

pub use crate::ctypes_enums::*;

/// PDButtons come in groups of 3, so this is a convenience grouping of them.
#[derive(Debug, Copy, Clone)]
pub struct PDButtonsSet {
  pub current: PDButtons,
  pub pushed: PDButtons,
  pub released: PDButtons,
}
