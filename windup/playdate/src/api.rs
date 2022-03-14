use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::capi_state::CApiState;
use crate::ctypes_enums::*;
use crate::executor::Executor;
use crate::geometry::Vector3;
use crate::graphics::Graphics;
use crate::time::{HighResolutionTimer, TimeTicks, WallClockTime};
use crate::String;

#[derive(Debug)]
pub struct Api {
  pub system: System,
  pub graphics: Graphics,
}
impl Api {
  pub(crate) fn new(state: &'static CApiState) -> Api {
    Api {
      system: System::new(state),
      graphics: Graphics::new(state),
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

  // System Api notes. Everything in the CSystem Api is exposed here in a Rusty way except:
  // - formatString() is not exposed, as the format!() macro replaces it in Rust.
  // - setUpdateCallback() is not exposed, as it is used internally.
  // - drawFPS() is moved to the Graphics api.
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
  pub fn set_menu_image(&mut self, bitmap: &crate::graphics::LCDBitmap, xoffset: i32) {
    // SAFETY: Playdate makes a copy from the given pointer, so we can pass it in and then drop the
    // reference on `bitmap` when we leave the function.
    let ptr = unsafe { bitmap.get_mut_ptr() };
    unsafe { self.state.csystem.setMenuImage.unwrap()(ptr, xoffset.clamp(0, 200)) }
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
  pub async fn next(&self) -> Inputs {
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
  type Output = Inputs;

  fn poll(self: Pin<&mut Self>, ctxt: &mut Context<'_>) -> Poll<Inputs> {
    let frame = self.state.frame_number.get();

    if frame > self.seen_frame {
      Poll::Ready(Inputs {
        state: self.state,
        frame_number: frame,
        peripherals_enabled: self.state.peripherals_enabled.get(),
      })
    } else {
      // Register the waker to be woken when the frame changes. We will observe that it has
      // indeed changed and return Ready since we have saved the current frame at construction.
      Executor::add_waker_for_update_callback(self.state.executor.as_ptr(), ctxt.waker());
      Poll::Pending
    }
  }
}

#[derive(Debug)]
pub struct Inputs {
  state: &'static CApiState,
  frame_number: u64,
  peripherals_enabled: PDPeripherals,
}
impl Inputs {
  /// The current frame number, which is monotonically increasing after the return of each call to
  /// `FrameWatcher::next()`
  pub fn frame_number(&self) -> u64 {
    self.frame_number
  }

  /// Returns the last read values from the accelerometor.
  ///
  /// These values are only present if the accelerometer is enabled via `System::enable_devices()`,
  /// otherwise it returns None.
  pub fn accelerometer(&self) -> Option<Vector3<f32>> {
    if self.peripherals_enabled.0 & PDPeripherals::kAccelerometer.0 != 0 {
      let mut v = Vector3::default();
      unsafe { self.state.csystem.getAccelerometer.unwrap()(&mut v.x, &mut v.y, &mut v.z) }
      Some(v)
    } else {
      None
    }
  }
}
pub struct Error(pub String);

impl AsRef<str> for Error {
  fn as_ref(&self) -> &str {
    &self.0
  }
}
