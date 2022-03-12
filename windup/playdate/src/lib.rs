#![no_std]
#![deny(clippy::all)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

extern crate playdate_macro;

/// A game crate should annotate their game loop function with this attribute macro.
/// 
/// The annotated function must be async, and will indicate that it's done updating
/// and ready to draw by `await`ing the `draw` Future passed to it.
pub use playdate_macro::main;

mod allocator;
mod cstring;
#[doc(hidden)]
pub mod macro_helpers;

pub use cstring::{CStr, CString};

#[global_allocator]
pub static GLOBAL_ALLOCATOR: allocator::Allocator = allocator::Allocator::new();

/// A helper implementation of panic_handler for the toplevel crate to forward to.
///
/// Since the top-level crate has to implement the `#[panic_handler]` we make it
/// easy by letting them simply forward over to this function.
pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
  // TODO: Dump a log somewhere?
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
