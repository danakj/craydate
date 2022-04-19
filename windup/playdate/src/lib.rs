#![no_std]
#![deny(clippy::all)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(never_type)]

extern crate alloc;
extern crate playdate_macro;

/// A game crate should annotate their game loop function with this attribute macro.
///
/// The annotated function must be async, and will indicate that it's done updating
/// and ready to draw by `await`ing the `draw` Future passed to it.
pub use playdate_macro::main;

mod allocator;
mod api;
mod bitmap;
mod callbacks;
mod capi_state;
mod color;
mod ctypes;
mod ctypes_enums;
mod debug;
mod display;
mod error;
mod executor;
mod file;
mod font;
mod geometry;
mod graphics;
mod inputs;
mod menu;
mod null_terminated;
mod sound;
mod system_event;
mod time;
mod video;

#[doc(hidden)]
pub mod macro_helpers;

/// Reexport some of alloc, since things in alloc are not guaranteed to work in `no_std` as it all
/// depends on our global allocator. This makes it clear they can be used, and avoids the need for
/// `export mod alloc` elsewhere.
pub use alloc::{borrow::ToOwned, format, string::String};

pub use api::*;
pub use bitmap::*;
pub use callbacks::{CallbackBuilder, Callbacks};
pub use color::*;
pub use ctypes_enums::*;
pub use display::*;
pub use error::*;
pub use file::*;
pub use font::*;
pub use geometry::*;
pub use graphics::*;
pub use inputs::*;
pub use menu::*;
pub use sound::*;
pub use system_event::*;
pub use time::{SoundTicks, TimeDelta, TimeTicks};
pub use video::*;

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: allocator::Allocator = allocator::Allocator::new();

/// A helper implementation of panic_handler for the toplevel crate to forward to.
///
/// Since the top-level crate has to implement the `#[panic_handler]` we make it
/// easy by letting them simply forward over to this function.
pub fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
  crate::debug::log_to_stdout("panic!");
  if let Some(loc) = panic_info.location() {
    crate::debug::log_to_stdout(" at ");
    crate::debug::log_to_stdout(loc.file());
    crate::debug::log_to_stdout(":");
    crate::debug::log_usize_to_stdout(loc.line() as usize);
    crate::debug::log_to_stdout(":");
    crate::debug::log_usize_to_stdout(loc.column() as usize);

    // TODO: caller()s.

    crate::debug::log_to_stdout_with_newline("");
  }

  if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
    crate::debug::log_to_stdout("payload: ");
    crate::debug::log_to_stdout(s);
    crate::debug::log_to_stdout("\n");
  } else {
    //crate::debug::log_bytes_to_stdout(b"panic has unknown payload");
  }

  core::intrinsics::abort()
}

#[alloc_error_handler]
pub fn my_example_handler(layout: core::alloc::Layout) -> ! {
  panic!(
    "memory allocation of {} bytes at alignment {} failed",
    layout.size(),
    layout.align()
  )
}
/// A way to store a pointer in a static variable, by telling the compiler it's Sync.
///
/// This is, of course, unsound if the pointer is used across threads and is not
/// thread-safe, but the pointer is only used by the Playdate system.
#[repr(transparent)]
struct BssPtr(*const u32);
unsafe impl Sync for BssPtr {}

extern "C" {
  static __bss_start__: u32;
  static __bss_end__: u32;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[used]
#[link_section = ".bss_start"]
static BSS_START_PTR: BssPtr = unsafe { BssPtr(&__bss_start__) };

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[used]
#[link_section = ".bss_end"]
static BSS_END_PTR: BssPtr = unsafe { BssPtr(&__bss_end__) };
