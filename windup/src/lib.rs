#![no_std]
#![feature(never_type)]

use playdate::CStr;
use playdate::LCDColor;
use playdate::LCDPattern;

#[playdate::main]
async fn main(api: playdate::Api) -> ! {
  let system = &api.system;
  let graphics = &api.graphics;

  let grey50: LCDPattern = [
    // Bitmap
    0b10101010, 0b01010101, 0b10101010, 0b01010101, 0b10101010, 0b01010101, 0b10101010, 0b01010101,
    // Mask
    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
  ];

  graphics.clear(LCDColor::Pattern(&grey50));
  loop {
    let fw = system.frame_watcher();
    system.log(CStr::from_bytes_with_nul(b"before\0").unwrap());
    fw.next().await;
    system.log(CStr::from_bytes_with_nul(b"after\0").unwrap());
  }
}
