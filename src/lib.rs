use playdate_sys::PlaydateAPI;
use playdate_sys::PDSystemEvent;
use std::ffi::CString;


#[no_mangle]
pub extern "C" fn eventHandler(api: *mut PlaydateAPI, event: PDSystemEvent, _arg: u32) -> ::std::os::raw::c_int {

    let cstr = CString::new("hello world maybe").unwrap();
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