#![no_std]
#![deny(clippy::all)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

extern crate playdate_macro;

pub mod prelude {
  // Macros and traits should be re-exported in here, as well as very common types that
  // should always be available without their full path.

  pub use crate::cstring::{CStr, CString};
}
// The prelude section is also used in this crate.
use prelude::*;

pub use playdate_macro::main;

mod allocator;
mod cstring;
mod event_loop;
#[doc(hidden)]
pub mod macro_helpers;

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
