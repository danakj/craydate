//! This module re-exports playdate_sys types with more consistent names.

pub use playdate_sys::playdate_graphics as CGraphics;
pub use playdate_sys::playdate_sys as CSystem;
pub use playdate_sys::LCDBitmap as CLCDBitmap;
pub use playdate_sys::PDSystemEvent as CSystemEvent;
pub use playdate_sys::PlaydateAPI as CApi;

pub use crate::ctypes_enums::*;
