//! This module re-exports playdate_sys types with more consistent names.
pub use playdate_sys::playdate_display as CDisplay;
pub use playdate_sys::playdate_file as CFile;
pub use playdate_sys::playdate_graphics as CGraphics;
pub use playdate_sys::playdate_sys as CSystem;
pub use playdate_sys::playdate_video as CVideo;
pub use playdate_sys::FileStat as CFileStat;
pub use playdate_sys::LCDBitmap as CLCDBitmap;
pub use playdate_sys::LCDColor as CLCDColor;
pub use playdate_sys::LCDPattern as CLCDPattern;
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
