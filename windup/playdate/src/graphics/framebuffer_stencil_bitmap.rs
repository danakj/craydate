use super::bitmap::BitmapRef;
use crate::capi_state::CApiState;

/// A sentinel that marks a bitmap acting as the stencil for drawing. Destroying this object will
/// unset the bitmap as the stencil.
pub struct FramebufferStencilBitmap<'a> {
  generation: usize,
  bitmap: &'a BitmapRef,
}
impl<'a> FramebufferStencilBitmap<'a> {
  pub fn bitmap(&self) -> &'a BitmapRef {
    self.bitmap
  }

  pub(crate) fn fns() -> &'static playdate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}
impl Drop for FramebufferStencilBitmap<'_> {
  fn drop(&mut self) {
    // Use a generation tag to avoid unsetting the stencil if another bitmap was set before this
    // object was dropped.
    if self.generation == CApiState::get().stencil_generation.get() {
      unsafe { Self::fns().setStencil.unwrap()(core::ptr::null_mut()) }
    }
  }
}
