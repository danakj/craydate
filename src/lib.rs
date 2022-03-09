#![deny(clippy::all)]
#![no_std]

mod cstring;

use playdate_sys::PDSystemEvent;
use playdate_sys::PlaydateAPI;

#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> i32 {
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
