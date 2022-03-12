#![no_std]
use playdate::CStr;

#[no_mangle]
extern "C" fn playdate_setup() {}

#[no_mangle]
extern "C" fn playdate_loop() {}

#[playdate::main]
fn main() -> &'static CStr {
    CStr::from_bytes_with_nul(b"hello from main\0").unwrap()
}