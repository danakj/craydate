//! This module re-exports playdate_sys enums that are passed through directly.

pub use playdate_sys::LCDBitmapDrawMode;
pub use playdate_sys::LCDBitmapFlip;
pub use playdate_sys::LCDPolygonFillRule;
pub use playdate_sys::LCDSolidColor;
pub use playdate_sys::PDLanguage;
pub use playdate_sys::PDPeripherals;
pub use playdate_sys::PDStringEncoding;

pub const LCD_COLUMNS: u32 = playdate_sys::LCD_COLUMNS;
pub const LCD_ROWS: u32 = playdate_sys::LCD_ROWS;
pub const LCD_ROWBYTES: u32 = playdate_sys::LCD_ROWSIZE;
