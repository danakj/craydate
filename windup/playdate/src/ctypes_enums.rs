//! This module re-exports playdate_sys enums that are passed through directly. The content of
//! module is exposed publicly to the game.

pub use playdate_sys::LCDBitmapDrawMode as BitmapDrawMode;
pub use playdate_sys::LCDBitmapFlip as BitmapFlip;
pub use playdate_sys::LCDPolygonFillRule as PolygonFillRule;
pub use playdate_sys::LCDSolidColor as SolidColor;
pub use playdate_sys::PDLanguage as Language;
pub use playdate_sys::PDPeripherals as Peripherals;
pub use playdate_sys::SoundFormat as SoundFormat;
pub use playdate_sys::SoundWaveform as SoundWaveform;
pub use playdate_sys::TwoPoleFilterType as TwoPoleFilterType;

pub const LCD_COLUMNS: u32 = playdate_sys::LCD_COLUMNS;
pub const LCD_ROWS: u32 = playdate_sys::LCD_ROWS;
pub const LCD_ROWBYTES: u32 = playdate_sys::LCD_ROWSIZE;
