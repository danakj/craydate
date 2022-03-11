use crate::*;
use playdate_sys::{PDSystemEvent, PlaydateAPI};

#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> i32 {
    let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(b"hello world maybe\0") };
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
