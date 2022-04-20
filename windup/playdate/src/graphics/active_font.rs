use super::font::Font;
use crate::capi_state::CApiState;

/// A sentinel that marks a font as the currently active font. Destroying this object will
/// unset the font as current.
pub struct ActiveFont<'a> {
  generation: usize,
  font: &'a Font,
}
impl<'a> ActiveFont<'a> {
  pub fn font(&self) -> &'a Font {
    self.font
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}
impl Drop for ActiveFont<'_> {
  fn drop(&mut self) {
    // Use a generation tag to avoid unsetting the font if another font was set before this
    // object was dropped.
    if self.generation == CApiState::get().font_generation.get() {
      unsafe { Self::fns().setFont.unwrap()(core::ptr::null_mut()) }
    }
  }
}
