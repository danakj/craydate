use super::bitmap::BitmapRef;
use crate::ctypes::*;

/// Information about a single bitmap, used in testing for collision between two bitmaps opaque
/// pixels.
pub struct BitmapCollider<'a> {
  /// A bitmap being tested for collision.
  pub bitmap: &'a BitmapRef,
  /// If the bitmap is flipped along each axis.
  pub flipped: BitmapFlip,
  /// The bitmap's x position.
  pub x: i32,
  /// The bitmap's y position.
  pub y: i32,
}
