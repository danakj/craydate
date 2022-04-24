mod active_font;
mod bitmap;
mod bitmap_collider;
mod color;
mod font;
mod framebuffer_stencil_bitmap;
mod graphics;
mod bitmap_data;
mod unowned_bitmap;
mod video;

pub use active_font::ActiveFont;
pub use bitmap::*;
pub use bitmap_data::BitmapData;
pub use bitmap_collider::BitmapCollider;
pub use color::{Color, Pattern, PixelColor};
pub use font::{Font, FontGlyph, FontPage};
pub use framebuffer_stencil_bitmap::FramebufferStencilBitmap;
pub use graphics::Graphics;
pub use unowned_bitmap::{UnownedBitmapRef, UnownedBitmapMut};
pub use video::Video;

use crate::ctypes::*;

fn playdate_rect_from_euclid(e: euclid::default::Rect<i32>) -> CLCDRect {
  CLCDRect {
    left: e.origin.x,
    top: e.origin.y,
    right: e.origin.x + e.size.width - 1,
    bottom: e.origin.y + e.size.height - 1,
  }
}
