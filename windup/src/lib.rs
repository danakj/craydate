#![no_std]
#![feature(never_type)]

use playdate::CStr;

#[playdate::main]
async fn main(api: playdate::Api) -> ! {
    let system = &api.system;
    loop {
        system.log(CStr::from_bytes_with_nul(b"before\0").unwrap());
        system.next_update().await;
        system.log(CStr::from_bytes_with_nul(b"after\0").unwrap());
    }
}