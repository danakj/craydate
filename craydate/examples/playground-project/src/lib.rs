#![deny(clippy::all)]
#![no_std]

extern crate game;

#[cfg(not(doc))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  // The #[panic_handler] must be in the top level crate, but we foward to the craydate
  // implementation.
  craydate::panic_handler(info)
}

// This provides symbols for compiler builtins, such as memcpy.
#[cfg(all(target_os = "windows"))]
#[link(name = "msvcrt")]
extern "C" {}
