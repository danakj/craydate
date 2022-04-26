mod active_font;
mod bitmap;
mod bitmap_collider;
mod bitmap_data;
mod color;
mod context_stack;
mod font;
mod framebuffer_stencil_bitmap;
mod graphics;
mod unowned_bitmap;
mod video;

pub(crate) use context_stack::ContextStack;

pub use active_font::ActiveFont;
pub use bitmap::*;
pub use bitmap_collider::BitmapCollider;
pub use bitmap_data::BitmapData;
pub use color::{Color, Pattern, PixelColor};
pub use context_stack::ContextStackId;
pub use font::{Font, FontGlyph, FontPage};
pub use framebuffer_stencil_bitmap::FramebufferStencilBitmap;
pub use graphics::Graphics;
pub use unowned_bitmap::{UnownedBitmapMut, UnownedBitmapRef};
pub use video::Video;

use crate::ctypes::*;

fn craydate_rect_from_euclid(e: euclid::default::Rect<i32>) -> CLCDRect {
  CLCDRect {
    left: e.origin.x,
    top: e.origin.y,
    right: e.origin.x + e.size.width - 1,
    bottom: e.origin.y + e.size.height - 1,
  }
}
