use super::bitmap::BitmapRef;
use crate::ctypes::*;

pub struct BitmapCollider<'a> {
    pub bitmap: &'a BitmapRef,
    pub flipped: BitmapFlip,
    pub x: i32,
    pub y: i32,
  }
  