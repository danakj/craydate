#![no_std]
#![deny(clippy::all)]
#![feature(core_intrinsics)]
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

#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> i32 {
    let cstr = unsafe { cstring::CStr::from_bytes_with_nul_unchecked(b"hello world maybe\0") };
    unsafe {
        let system = *(*api).system;
        system.logToConsole.unwrap()(cstr.as_ptr());
    }

    if event == PDSystemEvent::kEventInit {
        unsafe {
            GLOBAL_ALLOCATOR.set_system_ptr((*api).system as *mut playdate_sys::playdate_sys)
        };

        // set update callback here.
    }

    // send me $5 and I'll comment yikes on your undocumented return value
    0
}

//#[cfg(all(target_arch = "arm", target_os = "none"))]
type EventHandlerFn = extern "C" fn(*mut PlaydateAPI, PDSystemEvent, u32) -> i32;

//#[cfg(all(target_arch = "arm", target_os = "none"))]
#[used]
#[link_section = ".capi_handler"]
static EVENT_HANDLER: EventHandlerFn = eventHandler;
