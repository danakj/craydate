#![no_std]
#![deny(clippy::all)]
#![feature(never_type)]

use playdate::{LCDBitmapFlip, LCDColor, LCDPattern, LCDSolidColor, PDStringEncoding};

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

  let bmp = graphics.new_bitmap(100, 40, LCDColor::Solid(LCDSolidColor::kColorWhite));
  graphics.draw_bitmap(&bmp, 5, 9, LCDBitmapFlip::kBitmapUnflipped);
  drop(bmp);

  graphics.draw_text("Bloop", PDStringEncoding::kASCIIEncoding, 30, 20);

  let copy = graphics.copy_frame_buffer_bitmap();

  let mut data = copy.data();
  for i in 0..8*15 {
    data.pixels_mut().set(i, 0, false);
  }
  graphics.draw_bitmap(&copy, 0, 30, LCDBitmapFlip::kBitmapUnflipped);

  loop {
    let fw = system.frame_watcher();
    //system.log(CString::from_vec("cstring").unwrap());
    let s = playdate::String::new() + "before" + " with " + "concat";
    system.log(&s);
    fw.next().await;
    system.log("after");
  }
}
