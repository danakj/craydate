use core::cell::Cell;

use crate::capi_state::CApiState;
use crate::display::Display;
use crate::files::File;
use crate::graphics::Graphics;
use crate::sound::Sound;
use crate::time::{HighResolutionTimer, TimeTicks, WallClockTime};
use crate::{ctypes::*, SystemEventWatcher};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct System {
  // Runtime tracking to ensure only one timer is active.
  timer_active: Cell<bool>,
}
impl System {
  fn new() -> Self {
    System {
      timer_active: Cell::new(false),
    }
  }

  /// A watcher that lets you `await` for the next `SystemEvent`, such as the next frame with input
  /// events from the Playdate device.
  pub fn system_event_watcher(&self) -> SystemEventWatcher {
    SystemEventWatcher::new()
  }

  /// Returns the current time in milliseconds.
  pub fn current_time(&self) -> TimeTicks {
    TimeTicks::from_milliseconds(unsafe {
      CApiState::get().csystem.getCurrentTimeMilliseconds.unwrap()()
    })
  }

  /// Returns the current wall-clock time.
  ///
  /// This time is subject to drift and may go backwards. It can be useful when combined with
  /// timezone information for displaying a clock, but prefer `current_time()` for most application
  /// logic and for tracking elapsed time.
  pub fn wall_clock_time(&self) -> WallClockTime {
    let mut time = 0;
    unsafe { CApiState::get().csystem.getSecondsSinceEpoch.unwrap()(&mut time) };
    WallClockTime(time)
  }

  /// Starts a high resolution timer, and returns an object representing it.
  ///
  /// # Panics
  ///
  /// There can only be one HighResolutionTimer active at a time, as multiple timers would clobber
  /// each other inside Playdate. This function will panic if a HighResolutionTimer is started while
  /// another is active. Drop the returned HighResolutionTimer to finish using it.
  pub fn start_timer(&self) -> HighResolutionTimer {
    if self.timer_active.get() {
      panic!("HighResolutionTimer is already active.")
    }
    let timer = HighResolutionTimer::new(CApiState::get().csystem, &self.timer_active);
    unsafe { CApiState::get().csystem.resetElapsedTime.unwrap()() };
    timer
  }

  /// Returns whether the global "flipped" system setting is set.
  pub fn is_flipped_enabled(&self) -> bool {
    unsafe { CApiState::get().csystem.getFlipped.unwrap()() != 0 }
  }

  /// Returns whether the global "reduce flashing" system setting is set.
  pub fn is_reduce_flashing_enabled(&self) -> bool {
    unsafe { CApiState::get().csystem.getReduceFlashing.unwrap()() != 0 }
  }

  /// Returns the battery percentage, which is a value between 0 and 1.
  pub fn battery_percentage(&self) -> f32 {
    unsafe { CApiState::get().csystem.getBatteryPercentage.unwrap()() / 100f32 }
  }

  /// Returns the battery voltage.
  pub fn battery_voltage(&self) -> f32 {
    unsafe { CApiState::get().csystem.getBatteryVoltage.unwrap()() }
  }

  /// Sets the bitmap to be displayed beside (and behind) the system menu.
  ///
  /// The bitmap _must_ be 400x240 pixels, and an error will be logged if it is not. All important
  /// content should be in the left half of the image in an area 200 pixels wide, as the menu will
  /// obscure the rest. The right side of the image will be visible briefly as the menu animates in
  /// and out.
  ///
  /// The `xoffset` is clamped to between 0 and 200. If it is non-zero, the bitmap will be animated
  /// to the left by `xoffset` pixels. For example, if the offset is 200 then the right 200 pixels
  /// would be visible instead of the left 200 pixels while the menu is open.
  ///
  /// The bitmap will be copied, so the reference is not held.
  pub fn set_menu_image(&mut self, bitmap: &crate::graphics::BitmapRef, xoffset: i32) {
    // SAFETY: Playdate makes a copy from the given pointer, so we can pass it in and then drop the
    // reference on `bitmap` when we leave the function.
    unsafe {
      CApiState::get().csystem.setMenuImage.unwrap()(bitmap.as_bitmap_ptr(), xoffset.clamp(0, 200))
    }
  }

  /// Removes the user-specified bitmap from beside the system menu. The default image is displayed
  /// instead.
  pub fn clear_menu_image(&mut self) {
    unsafe { CApiState::get().csystem.setMenuImage.unwrap()(core::ptr::null_mut(), 0) }
  }

  /// To use a peripheral, it must first be enabled via this function.
  ///
  /// By default, the accelerometer is disabled to save (a small amount of) power. Once enabled,
  /// accelerometer data is not available until the next frame, and will be accessible from the
  /// output of `FrameWatcher::next()`.
  pub fn enable_peripherals(&mut self, which: Peripherals) {
    CApiState::get().peripherals_enabled.set(which);
    unsafe { CApiState::get().csystem.setPeripheralsEnabled.unwrap()(which) }
  }

  /// Returns the current language of the system.
  pub fn get_language(&self) -> Language {
    unsafe { CApiState::get().csystem.getLanguage.unwrap()() }
  }

  /// Disables or enables the 60 second auto-lock feature. When enabled, the timer is reset to 60
  /// seconds.
  ///
  /// As of 0.10.3, the device will automatically lock if the user doesn’t press any buttons or use
  /// the crank for more than 60 seconds. In order for games that expect longer periods without
  /// interaction to continue to function, it is possible to manually disable the auto lock feature.
  /// Note that when disabling the timeout, developers should take care to re-enable the timeout
  /// when appropiate.
  pub fn set_auto_lock(&mut self, val: AutoLock) {
    let disabled = match val {
      AutoLock::Disabled => 1,
      AutoLock::Enabled => 0,
    };
    unsafe { CApiState::get().csystem.setAutoLockDisabled.unwrap()(disabled) }
  }

  /// Disables or enables sound effects when the crank is docked or undocked.
  ///
  /// Playdate 0.12 adds sound effects for various system events, such as the menu opening or
  /// closing, USB cable plugged or unplugged, and the crank docked or undocked. Since games can
  /// receive notification of the crank docking and undocking, and may incorporate this into the
  /// game, Playdate provides a function for muting the default sounds for these events.
  ///
  /// # Return
  ///
  /// The function returns the previous value for this setting.
  pub fn set_crank_sounds(&mut self, val: CrankSounds) -> CrankSounds {
    let disabled = match val {
      CrankSounds::Silent => 1,
      CrankSounds::DockingSounds => 0,
    };
    let previous = unsafe { CApiState::get().csystem.setCrankSoundsDisabled.unwrap()(disabled) };
    match previous {
      0 => CrankSounds::DockingSounds,
      _ => CrankSounds::Silent,
    }
  }
}

/// The state of the auto-lock system.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AutoLock {
  /// The auto-lock is disabled. The device will not lock when idle.
  Disabled,
  /// The auto-lock is enabled, and will lock when idle.
  Enabled,
}

/// Whether using the crank makes sounds when docked or undocked.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CrankSounds {
  /// The crank is silent, in case the application wishes to provide their own sound effects.
  Silent,
  /// The crank makes sounds when docked or undocked.
  DockingSounds,
}
