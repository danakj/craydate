use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::display::Display;
use crate::executor::Executor;
use crate::file::File;
use crate::graphics::Graphics;
use crate::time::{HighResolutionTimer, TimeTicks, WallClockTime};
use crate::String;

#[derive(Debug)]
pub struct Api {
  pub system: System,
  pub display: Display,
  pub graphics: Graphics,
  pub file: File,
}
impl Api {
  pub(crate) fn new(state: &'static CApiState) -> Api {
    Api {
      system: System::new(state),
      display: Display::new(state),
      graphics: Graphics::new(state),
      file: File::new(state),
    }
  }
}

#[derive(Debug)]
pub struct System {
  pub(crate) state: &'static CApiState,
  // Runtime tracking to ensure only one timer is active.
  timer_active: Cell<bool>,
}
impl System {
  fn new(state: &'static CApiState) -> Self {
    System {
      state,
      timer_active: Cell::new(false),
    }
  }

  // System Api notes. Everything in the "Utility", "Device Auto Lock", and "System Sounds" api
  // sections is exposed here in a Rusty way except:
  // - formatString() is not exposed, as the format!() macro replaces it in Rust.
  // - setUpdateCallback() is not exposed, as it is used internally. The ability to wait for the
  //   next update (i.e. frame) is instead done through `frame_watcher()` that provides an async
  //   function that returns when the next update happens.
  // - drawFPS() is moved to the Graphics api.
  // - getLanguage() from Graphics > Miscellaneous is moved to here.
  // - TODO: All system menu interaction functions.

  /// A watcher that lets you `await` for the next frame update from the Playdate device.
  pub fn frame_watcher(&self) -> FrameWatcher {
    FrameWatcher { state: self.state }
  }

  /// Prints a string to the Playdate console, as well as to stdout.
  pub fn log<S: AsRef<str>>(&self, s: S) {
    crate::debug::log(s)
  }

  /// Prints an error string in red to the Playdate console, and pauses Playdate. Also prints the
  /// string to stdout.
  pub fn error<S: AsRef<str>>(&self, s: S) {
    crate::debug::error(s);
  }

  /// Returns the current time in milliseconds.
  pub fn current_time(&self) -> TimeTicks {
    TimeTicks(unsafe { self.state.csystem.getCurrentTimeMilliseconds.unwrap()() })
  }

  /// Returns the current wall-clock time.
  ///
  /// This time is subject to drift and may go backwards. It can be useful when combined with
  /// timezone information for displaying a clock, but prefer `current_time()` for most application
  /// logic and for tracking elapsed time.
  pub fn wall_clock_time(&self) -> WallClockTime {
    let mut time = 0;
    unsafe { self.state.csystem.getSecondsSinceEpoch.unwrap()(&mut time) };
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
    let timer = HighResolutionTimer::new(self.state.csystem, &self.timer_active);
    unsafe { self.state.csystem.resetElapsedTime.unwrap()() };
    timer
  }

  /// Returns whether the global "flipped" system setting is set.
  pub fn is_flipped_enabled(&self) -> bool {
    unsafe { self.state.csystem.getFlipped.unwrap()() != 0 }
  }

  /// Returns whether the global "reduce flashing" system setting is set.
  pub fn is_reduce_flashing_enabled(&self) -> bool {
    unsafe { self.state.csystem.getReduceFlashing.unwrap()() != 0 }
  }

  /// Returns the battery percentage, which is a value between 0 and 1.
  pub fn battery_percentage(&self) -> f32 {
    unsafe { self.state.csystem.getBatteryPercentage.unwrap()() / 100f32 }
  }

  /// Returns the battery voltage.
  pub fn battery_voltage(&self) -> f32 {
    unsafe { self.state.csystem.getBatteryVoltage.unwrap()() }
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
  pub fn set_menu_image(&mut self, bitmap: &crate::bitmap::BitmapRef, xoffset: i32) {
    // SAFETY: Playdate makes a copy from the given pointer, so we can pass it in and then drop the
    // reference on `bitmap` when we leave the function.
    unsafe {
      self.state.csystem.setMenuImage.unwrap()(bitmap.as_bitmap_ptr(), xoffset.clamp(0, 200))
    }
  }

  /// Removes the user-specified bitmap from beside the system menu. The default image is displayed
  /// instead.
  pub fn clear_menu_image(&mut self) {
    unsafe { self.state.csystem.setMenuImage.unwrap()(core::ptr::null_mut(), 0) }
  }

  /// To use a peripheral, it must first be enabled via this function.
  ///
  /// By default, the accelerometer is disabled to save (a small amount of) power. Once enabled,
  /// accelerometer data is not available until the next frame, and will be accessible from the
  /// output of `FrameWatcher::next()`.
  pub fn enable_peripherals(&mut self, which: PDPeripherals) {
    self.state.peripherals_enabled.set(which);
    unsafe { self.state.csystem.setPeripheralsEnabled.unwrap()(which) }
  }

  /// Returns the current language of the system.
  pub fn get_language(&self) -> Language {
    unsafe { self.state.csystem.getLanguage.unwrap()() }
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
    unsafe { self.state.csystem.setAutoLockDisabled.unwrap()(disabled) }
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
    let previous = unsafe { self.state.csystem.setCrankSoundsDisabled.unwrap()(disabled) };
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

#[derive(Debug)]
pub struct FrameWatcher {
  state: &'static CApiState,
}
impl FrameWatcher {
  /// Runs until the next frame from the Playdate device, then returns the frame number.
  ///
  /// This function returns after the Playdate device calls the "update callback" to signify that
  /// the game should perform updates for the next frame to be displayed.
  pub async fn next(&self) -> crate::inputs::Inputs {
    self.next_impl().await
  }
  fn next_impl(&self) -> FrameWatcherFuture {
    FrameWatcherFuture {
      state: self.state,
      seen_frame: self.state.frame_number.get(),
    }
  }
}

/// A future for which poll() waits for the next update, then returns Complete.
struct FrameWatcherFuture {
  state: &'static CApiState,
  seen_frame: u64,
}

impl Future for FrameWatcherFuture {
  type Output = crate::inputs::Inputs;

  fn poll(self: Pin<&mut Self>, ctxt: &mut Context<'_>) -> Poll<crate::inputs::Inputs> {
    let frame = self.state.frame_number.get();

    if frame > self.seen_frame {
      if frame > self.seen_frame + 1 {
        crate::debug::log(
          "WARNING: FrameWatcher missed a frame. This could happen if an async function was called 
          and `await`ed without also waiting for the FrameWatcher via select(). Currently only one
          async function can run (we don't support a spawn()) and thus we don't have a select(). So 
          if this occurs it's a surprising bug.",
        )
      }

      Poll::Ready(crate::inputs::Inputs::new(
        self.state,
        frame,
        self.state.peripherals_enabled.get(),
        &self.state.button_state_per_frame.get().map(|b| b.unwrap()),
      ))
    } else {
      // Register the waker to be woken when the frame changes. We will observe that it has
      // indeed changed and return Ready since we have saved the current frame at construction.
      Executor::add_waker_for_update_callback(self.state.executor.as_ptr(), ctxt.waker());
      Poll::Pending
    }
  }
}

pub struct Error(pub String);

impl AsRef<str> for Error {
  fn as_ref(&self) -> &str {
    &self.0
  }
}
impl From<String> for Error {
  fn from(s: String) -> Self {
    Error(s)
  }
}
impl From<&str> for Error {
  fn from(s: &str) -> Self {
    Error(s.into())
  }
}
impl From<&mut str> for Error {
  fn from(s: &mut str) -> Self {
    Error(s.into())
  }
}
impl core::fmt::Debug for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Error({})", self.0)
  }
}
impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.0)
  }
}
