extern crate alloc; // `alloc` is fine to use once eventHandler() has set up the allocator.

use crate::*;
use alloc::boxed::Box;
use core::ffi::c_void;
use playdate_sys::{PDSystemEvent, PlaydateAPI};

#[derive(Debug, Default)]
struct UpdateCallbackData {
    _i: i32,
}

/// The main event loop for the Playdate game.
/// 
/// # Return
/// 
/// Return a non-zero value if the screen should be updated. Otherwise, 0 to indicate
/// there is nothing to draw.
extern "C" fn update_callback(_data: *mut c_void) -> i32 {
    0
}

/// Set up the execution environment for the Playdate device.
///
/// This function is called twice during the initialization of the Playdate device. The first
/// time with the event `kEventInit`. In order to keep control of the game in Rust, the
/// `kEventInit` event must set an update callback. Otherwise, it is called second time with
/// the event `kEventInitLua` and then attempts to execute a lua program.
#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> i32 {
    let system = unsafe { *(*api).system };
    if event == PDSystemEvent::kEventInit {
        unsafe {
            GLOBAL_ALLOCATOR.set_system_ptr((*api).system as *mut playdate_sys::playdate_sys)
        };

        let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(b"hello world maybe\0") };
        unsafe { system.logToConsole.unwrap()(cstr.as_ptr()) };

        // We will leak this UpdateCallbackData pointer so it has 'static lifetime.
        let data_ptr = Box::into_raw(Box::new(UpdateCallbackData::default())) as *mut c_void;
        unsafe { system.setUpdateCallback.unwrap()(Some(update_callback), data_ptr) };
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
