#![no_std]
#![deny(clippy::all)]
#![feature(never_type)]

use playdate::{format, LCDBitmapFlip, LCDPattern, LCDSolidColor, PDStringEncoding};

#[playdate::main]
async fn main(mut api: playdate::Api) -> ! {
  let system = &api.system;
  let graphics = &api.graphics;

  let grey50: LCDPattern = [
    // Bitmap
    0b10101010, 0b01010101, 0b10101010, 0b01010101, 0b10101010, 0b01010101, 0b10101010, 0b01010101,
    // Mask
    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
  ];
  graphics.clear(&grey50);

  let bmp = graphics.new_bitmap(100, 40, LCDSolidColor::kColorWhite);
  graphics.draw_bitmap(&bmp, 5, 9, LCDBitmapFlip::kBitmapUnflipped);
  drop(bmp);

  graphics.draw_text("Bloop", PDStringEncoding::kASCIIEncoding, 30, 20);

  let copy = graphics.copy_frame_buffer_bitmap();

  let mut data = copy.data();
  for y in 20..30 {
    for x in 10..20 {
      data.pixels_mut().set(x, y, false);
    }
  }
  graphics.draw_bitmap(&copy, 0, 30, LCDBitmapFlip::kBitmapUnflipped);

  // working image
  let yo_path = "images/yo";
  let load = graphics.load_bitmap(yo_path);
  if let Ok(bitmap) = load {
    graphics.draw_bitmap(&bitmap, 100, 80, LCDBitmapFlip::kBitmapUnflipped);
  }

  // broken image
  let broken_path = "images/wat";
  let load = graphics.load_bitmap(broken_path);
  if let Err(error) = load {
    system.log(error);
  }

  let display = &mut api.display;
  display.set_inverted(true);
  display.set_flipped(true, false);
  display.set_scale(2);

  system.log(format!(
    "Entering main loop at time {}",
    api.system.current_time()
  ));
  let fw = system.frame_watcher();
  loop {
    let inputs = fw.next().await;
    for (button, event) in inputs.buttons().all_events() {
      match event {
        playdate::ButtonEvent::Push => {
          api.system.log(format!(
            "{:?} pushed on frame {}",
            button,
            inputs.frame_number()
          ));
        }
        playdate::ButtonEvent::Release => {
          api.system.log(format!(
            "{:?} released on frame {}",
            button,
            inputs.frame_number()
          ));
        }
      }
    }

    api.graphics.draw_fps(400 - 15, 0);
  }
}
