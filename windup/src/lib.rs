#![no_std]
#![feature(never_type)]

use playdate::CStr;

#[playdate::main]
async fn main(api: playdate::System) -> ! {
    loop {
        api.log(CStr::from_bytes_with_nul(b"before\0").unwrap());
        api.next_update().await;
        api.log(CStr::from_bytes_with_nul(b"after\0").unwrap());
    }
}