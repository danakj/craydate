//! This module re-exports playdate_sys enums that are passed through directly. The content of
//! module is exposed publicly to the game.

pub use craydate_sys::LCDBitmapDrawMode as BitmapDrawMode;
pub use craydate_sys::LCDBitmapFlip as BitmapFlip;
pub use craydate_sys::LCDPolygonFillRule as PolygonFillRule;
pub use craydate_sys::LCDSolidColor as SolidColor;
pub use craydate_sys::PDLanguage as Language;
pub use craydate_sys::PDPeripherals as Peripherals;
pub use craydate_sys::SoundFormat as SoundFormat;
pub use craydate_sys::SoundWaveform as SoundWaveform;
pub use craydate_sys::TwoPoleFilterType as TwoPoleFilterType;

pub const LCD_COLUMNS: u32 = craydate_sys::LCD_COLUMNS;
pub const LCD_ROWS: u32 = craydate_sys::LCD_ROWS;
pub const LCD_ROWBYTES: u32 = craydate_sys::LCD_ROWSIZE;
