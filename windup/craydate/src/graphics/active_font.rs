use super::font::Font;
use crate::capi_state::CApiState;

/// A sentinel that marks a font as the currently active font. Destroying this object will
/// unset the font as current.
pub struct ActiveFont<'a> {
  generation: usize,
  font: &'a Font,
}
impl<'a> ActiveFont<'a> {
  pub(crate) fn new(font: &'a Font) -> Self {
    // Track the generation number so as to only unset the font on drop if another font wasn't set
    // as active since.
    let generation = CApiState::get().font_generation.get() + 1;
    CApiState::get().font_generation.set(generation);
    ActiveFont { generation, font }
  }

  /// Returns the font that was set active when this object was constructed.
  pub fn font(&self) -> &'a Font {
    self.font
  }

  pub(crate) fn fns() -> &'static craydate_sys::playdate_graphics {
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
