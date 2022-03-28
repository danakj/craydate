use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::geometry::Vector3;

#[derive(Debug)]
pub struct Inputs {
  state: &'static CApiState,
  frame_number: u64,
  peripherals_enabled: Peripherals,
  buttons: Buttons,
  crank: Crank,
}
impl Inputs {
  // Button states are cached from the previous frame in order to infer button events that
  // happened between frames. So they are passed in to Inputs from the cache instead of pulled
  // from the device here.
  pub(crate) fn new(
    state: &'static CApiState,
    frame_number: u64,
    peripherals_enabled: Peripherals,
    button_state_per_frame: &[PDButtonsSet; 2],
  ) -> Self {
    let crank = if unsafe { state.csystem.isCrankDocked.unwrap()() != 0 } {
      Crank::Docked
    } else {
      Crank::Undocked {
        angle: unsafe { state.csystem.getCrankAngle.unwrap()() },
        change: unsafe { state.csystem.getCrankChange.unwrap()() },
      }
    };

    Inputs {
      state,
      frame_number,
      peripherals_enabled,
      buttons: Buttons::new(button_state_per_frame),
      crank,
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
    if self.peripherals_enabled & Peripherals::kAccelerometer == Peripherals::kAccelerometer {
      let mut v = Vector3::default();
      unsafe { self.state.csystem.getAccelerometer.unwrap()(&mut v.x, &mut v.y, &mut v.z) }
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

/// The status of the crank input device.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Crank {
  /// When docked, the crank can not be used and as no position.
  Docked,
  Undocked {
    /// The position of the crank in degrees. The angle increases when moved clockwise.
    angle: f32,
    /// The change in position of the crank, in degrees, since the last frame. The angle increases
    /// when moved clockwise, so the change will be negative when moved counter-clockwise.
    change: f32,
  },
}

/// The set of input buttons.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Button {
  /// The up arrow on the directional pad.
  Up,
  /// The down arrow on the directional pad.
  Down,
  /// The left arrow on the directional pad.
  Left,
  /// The right arrow on the directional pad.
  Right,
  /// The B button.
  B,
  /// The A button.
  A,
}

/// The current state of a button, which indicates if the player is holding the button down or not.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonState {
  /// The button is being pressed. The active state of a button.
  Pushed,
  /// The button is not being pressed. The neutral state of a button.
  Released,
}

/// Events which describe changes in state for a button.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ButtonEvent {
  /// The button was pressed.
  ///
  /// It moved from a `Released` to a `Pushed` state.
  Push,
  /// The button stopped being pressed
  ///
  /// It moved from a `Pushed` to a `Released` state.
  Release,
}

/// The state of all buttons, along with changes since the last frame.
#[derive(Debug)]
pub struct Buttons {
  current: CButtons,
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
      up_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonUp),
      down_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonDown),
      left_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonLeft),
      right_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonRight),
      b_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonB),
      a_events: Self::compute_events(&button_state_per_frame, CButtons::kButtonA),
    }
  }

  /// Infer a sequence of events for a button between 2 frames by combining the button's last pushed
  /// state, current pushed state, and whether a push and/or release happened in between frames.
  fn compute_events(frames: &[PDButtonsSet; 2], button: CButtons) -> [Option<ButtonEvent>; 3] {
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

  /// Helper function to convert the Playdate API bitmask to the ButtonState enum for a single
  /// button.
  #[inline]
  fn current_state(&self, button: CButtons) -> ButtonState {
    if self.current & button != CButtons(0) {
      ButtonState::Pushed
    } else {
      ButtonState::Released
    }
  }

  /// Returns an iterator over all button events that occured since the last frame.
  ///
  /// This reports any buttons which were pressed or released. The device does not report the order
  /// in which buttons were interacted with, so the order of events across buttons is arbitrary.
  /// Events for a single button are in an accurate order.
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

  /// Returns an iterator over all button events, on the `Up` button, that occred since the last
  /// frame.
  pub fn up_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.up_events.iter().filter_map(move |o| *o)
  }
  /// Returns an iterator over all button events, on the `Down` button, that occred since the last
  /// frame.
  pub fn down_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.down_events.iter().filter_map(move |o| *o)
  }
  /// Returns an iterator over all button events, on the `Left` button, that occred since the last
  /// frame.
  pub fn left_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.left_events.iter().filter_map(move |o| *o)
  }
  /// Returns an iterator over all button events, on the `Right` button, that occred since the last
  /// frame.
  pub fn right_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.right_events.iter().filter_map(move |o| *o)
  }
  /// Returns an iterator over all button events, on the `B` button, that occred since the last
  /// frame.
  pub fn b_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.b_events.iter().filter_map(move |o| *o)
  }
  /// Returns an iterator over all button events, on the `A` button, that occred since the last
  /// frame.
  pub fn a_events(&self) -> impl Iterator<Item = ButtonEvent> + '_ {
    self.a_events.iter().filter_map(move |o| *o)
  }

  /// Returns the current state of the `Up` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn up_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonUp)
  }
  /// Returns the current state of the `Down` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn down_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonDown)
  }
  /// Returns the current state of the `Left` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn left_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonLeft)
  }
  /// Returns the current state of the `Right` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn right_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonRight)
  }
  /// Returns the current state of the `B` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn b_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonB)
  }
  /// Returns the current state of the `A` button.
  ///
  /// Prefer to use the events functions to track button press and release, as this function would
  /// miss push+release sequences that are faster than a single frame.
  pub fn a_state(&self) -> ButtonState {
    self.current_state(CButtons::kButtonA)
  }
}
