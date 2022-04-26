use super::button_event::ButtonEvent;
use super::button::Button;
use super::button_state::ButtonState;
use crate::ctypes::*;


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
  pub(crate) fn new(button_state_per_frame: &[PDButtonsSet; 2]) -> Self {
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

  /// Helper function to convert the Playdate C Api bitmask to the ButtonState enum for a single
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
