use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::geometry::Vector3;

#[derive(Debug)]
pub struct Inputs {
  state: &'static CApiState,
  frame_number: u64,
  peripherals_enabled: PDPeripherals,
  // The button state for the current and previous frame, respectively.
  button_state_per_frame: [PDButtonsSet; 2],
}
impl Inputs {
  pub(crate) fn new(
    state: &'static CApiState,
    frame_number: u64,
    peripherals_enabled: PDPeripherals,
    button_state_per_frame: [PDButtonsSet; 2],
  ) -> Self {
    Inputs {
      state,
      frame_number,
      peripherals_enabled,
      button_state_per_frame,
    }
  }
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
    if self.peripherals_enabled & PDPeripherals::kAccelerometer == PDPeripherals::kAccelerometer {
      let mut v = Vector3::default();
      unsafe { self.state.csystem.getAccelerometer.unwrap()(&mut v.x, &mut v.y, &mut v.z) }
      Some(v)
    } else {
      None
    }
  }

  pub fn buttons(&self) -> Buttons {
    Buttons::new(&self.button_state_per_frame)
  }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Button {
  Up,
  Down,
  Left,
  Right,
  B,
  A,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonState {
  Pushed,
  Released,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonEvent {
  Push,
  Release,
}

/// The state of all buttons, along with changes since the last frame.
pub struct Buttons {
  current: PDButtons,
  up_events: [Option<ButtonEvent>; 3],
  down_events: [Option<ButtonEvent>; 3],
  left_events: [Option<ButtonEvent>; 3],
  right_events: [Option<ButtonEvent>; 3],
  b_events: [Option<ButtonEvent>; 3],
  a_events: [Option<ButtonEvent>; 3],
}
impl Buttons {
  fn new(button_state_per_frame: &[PDButtonsSet; 2]) -> Self {
    Buttons {
      current: button_state_per_frame[0].current,
      up_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonUp),
      down_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonDown),
      left_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonLeft),
      right_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonRight),
      b_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonB),
      a_events: Self::compute_events(&button_state_per_frame, PDButtons::kButtonA),
    }
  }

  /// Infer a sequence of events for a button between 2 frames by combining the button's last pushed
  /// state, current pushed state, and whether a push and/or release happened in between frames.
  fn compute_events(frames: &[PDButtonsSet; 2], button: PDButtons) -> [Option<ButtonEvent>; 3] {
    // Last frame: pushed.
    if frames[1].current & button == button {
      // Last frame: pushed. || Current frame: [released].
      if frames[0].current & button != button {
        // Was pushed between frames.
        if frames[0].pushed & button == button {
          // pushed || released -> pushed -> [released]
          [
            Some(ButtonEvent::Release),
            Some(ButtonEvent::Push),
            Some(ButtonEvent::Release),
          ]
        }
        // Was not pushed between frames.
        else {
          // pushed || [released]
          [Some(ButtonEvent::Release), None, None]
        }
      }
      // Last frame: pushed. || Current frame: [pushed].
      else {
        // Was pushed between frames.
        if frames[0].pushed & button == button {
          // pushed || released -> [pushed]
          [Some(ButtonEvent::Release), Some(ButtonEvent::Push), None]
        }
        // Was not pushed between frames.
        else {
          // [pushed] ||
          [None, None, None]
        }
      }
    }
    // Last frame: released.
    else {
      // Last frame: released. || Current frame: [pushed]
      if frames[0].current & button == button {
        // Was released between frames.
        if frames[0].released & button == button {
          // released || pushed -> released -> [pushed]
          [
            Some(ButtonEvent::Push),
            Some(ButtonEvent::Release),
            Some(ButtonEvent::Push),
          ]
        }
        // Was not released between frames.
        else {
          // released || pushed
          [Some(ButtonEvent::Push), None, None]
        }
      }
      // Last frame: released. || Current frame: [released].
      else {
        // Was released between frames.
        if frames[0].released & button == button {
          // released || pushed -> [released]
          [Some(ButtonEvent::Push), Some(ButtonEvent::Release), None]
        }
        // Was not released between frames.
        else {
          // [released] ||
          [None, None, None]
        }
      }
    }
  }

  #[inline]
  fn current_state(&self, button: PDButtons) -> ButtonState {
    if self.current & button != PDButtons(0) {
      ButtonState::Pushed
    } else {
      ButtonState::Released
    }
  }

  pub fn all_events(&self) -> impl Iterator<Item = (Button, ButtonEvent)> + '_ {
    self
      .up_events()
      .map(|e| (Button::Up, e))
      .chain(self.down_events().map(|e| (Button::Down, e)))
      .chain(self.left_events().map(|e| (Button::Left, e)))
      .chain(self.right_events().map(|e| (Button::Right, e)))
      .chain(self.b_events().map(|e| (Button::B, e)))
      .chain(self.a_events().map(|e| (Button::A, e)))
  }

  pub fn up_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.up_events.iter().filter_map(move |o| *o)
  }
  pub fn down_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.down_events.iter().filter_map(move |o| *o)
  }
  pub fn left_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.left_events.iter().filter_map(move |o| *o)
  }
  pub fn right_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.right_events.iter().filter_map(move |o| *o)
  }
  pub fn b_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.b_events.iter().filter_map(move |o| *o)
  }
  pub fn a_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.a_events.iter().filter_map(move |o| *o)
  }

  pub fn up_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonUp)
  }
  pub fn down_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonDown)
  }
  pub fn left_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonLeft)
  }
  pub fn right_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonRight)
  }
  pub fn b_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonB)
  }
  pub fn a_state(&self) -> ButtonState {
    self.current_state(PDButtons::kButtonA)
  }
}
