/// Metadata for an `Bitmap`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitmapData {
  width: i32,
  height: i32,
  rowbytes: i32,
  hasmask: i32,
}
impl BitmapData {
  pub(crate) fn new(width: i32, height: i32, rowbytes: i32, hasmask: i32) -> Self {
    Self {
      width,
      height,
      rowbytes,
      hasmask,
    }
  }

  /// The number of pixels (or, columns) per row of the bitmap.
  ///
  /// Each pixel is a single bit, and there may be more bytes (as determined by `row_bytes()`) in a
  /// row than required to hold all the pixels.
  pub fn width(&self) -> i32 {
    self.width
  }
  /// The number of rows in the bitmap.
  pub fn height(&self) -> i32 {
    self.height
  }
  /// The number of bytes per row of the bitmap.
  pub fn row_bytes(&self) -> i32 {
    self.rowbytes
  }
  /// Whether the bitmap has a mask attached, via `set_mask_bitmap()`.
  pub fn has_mask(&self) -> bool {
    self.hasmask != 0
  }
}
