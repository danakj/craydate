use super::buttons::Buttons;
use super::crank::Crank;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::geometry::Vector3;
use crate::system::System;

/// The set of all input state and/or changes since the last frame.
#[derive(Debug)]
pub struct Inputs {
  peripherals_enabled: Peripherals,
  buttons: Buttons,
  crank: Crank,
}
impl Inputs {
  // Button states are cached from the previous frame in order to infer button events that
  // happened between frames. So they are passed in to Inputs from the cache instead of pulled
  // from the device here.
  pub(crate) fn new(
    peripherals_enabled: Peripherals,
    button_state_per_frame: &[PDButtonsSet; 2],
  ) -> Self {
    let state = CApiState::get();
    let crank = if unsafe { state.csystem.isCrankDocked.unwrap()() != 0 } {
      Crank::Docked
    } else {
      Crank::Undocked {
        angle: unsafe { state.csystem.getCrankAngle.unwrap()() },
        change: unsafe { state.csystem.getCrankChange.unwrap()() },
      }
    };

    Inputs {
      peripherals_enabled,
      buttons: Buttons::new(button_state_per_frame),
      crank,
    }
  }

  /// Returns the last read values from the accelerometor.
  ///
  /// These values are only present if the accelerometer is enabled via `System::enable_devices()`,
  /// otherwise it returns None.
  pub fn accelerometer(&self) -> Option<Vector3<f32>> {
    if self.peripherals_enabled & Peripherals::kAccelerometer == Peripherals::kAccelerometer {
      let mut v = Vector3::default();
      unsafe { System::fns().getAccelerometer.unwrap()(&mut v.x, &mut v.y, &mut v.z) }
      Some(v)
    } else {
      None
    }
  }

  /// Returns the state of and events that occured since the last frame for all buttons.
  pub fn buttons(&self) -> &Buttons {
    &self.buttons
  }

  /// Returns the state of and change that occured since the last frame for the crank.
  pub fn crank(&self) -> &Crank {
    &self.crank
  }
}
