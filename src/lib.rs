#![deny(clippy::all)]
#![feature(core_intrinsics)]
#![no_std]
#![feature(default_alloc_error_handler)]

mod allocator;
mod cstring;

use playdate_sys::PDSystemEvent;
use playdate_sys::PlaydateAPI;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: allocator::Allocator = allocator::Allocator::new();

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    // TODO: Dump a log somewhere?
    core::intrinsics::abort()
}

#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> i32 {
    unsafe { GLOBAL_ALLOCATOR.set_system_ptr((*api).system as *mut playdate_sys::playdate_sys) };

    let cstr = unsafe { cstring::CStr::from_bytes_with_nul_unchecked(b"hello world maybe\0") };
    unsafe {
        let system = *(*api).system;
        system.logToConsole.unwrap()(cstr.as_ptr());
    }

    if event == PDSystemEvent::kEventInit {
        // set event handler here.
    }

    // send me $5 and I'll comment yikes on your undocumented return value
    0
}
