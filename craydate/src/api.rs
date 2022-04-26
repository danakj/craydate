use crate::display::Display;
use crate::files::File;
use crate::system::System;
use crate::graphics::Graphics;
use crate::sound::Sound;

/// Apis used to access the Playdate device's display, sound, files, clock, menus, etc.
/// 
/// This type is passed as a parameter to the `#[main]` function of the game.
#[derive(Debug)]
#[non_exhaustive]
pub struct Api {
  pub system: System,
  pub display: Display,
  pub graphics: Graphics,
  pub file: File,
  pub sound: Sound,
}
impl Api {
  pub(crate) fn new() -> Api {
    Api {
      system: System::new(),
      display: Display::new(),
      graphics: Graphics::new(),
      file: File::new(),
      sound: Sound::new(),
    }
  }
}
