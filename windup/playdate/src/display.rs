use crate::capi_state::CApiState;

#[derive(Debug)]
pub struct Display {
  pub(crate) state: &'static CApiState,
}
impl Display {
  pub(crate) fn new(state: &'static CApiState) -> Self {
    Display { state }
  }

  /// Returns the height of the display, taking the current scale into account;
  /// e.g. if the scale is 2, this function returns 120 instead of 240.
  pub fn height(&self) -> i32 {
    unsafe { self.state.cdisplay.getHeight.unwrap()() }
  }

  /// Returns the width of the display, taking the current scale into account;
  /// e.g. if the scale is 2, this function returns 200 instead of 400.
  pub fn width(&self) -> i32 {
    unsafe { self.state.cdisplay.getWidth.unwrap()() }
  }

  /// If `inverted` is true, the frame buffer is drawn inverted--black instead of white.
  pub fn set_inverted(&mut self, inverted: bool) {
    // Yes, this function takes an integer??
    unsafe { self.state.cdisplay.setInverted.unwrap()(inverted as i32) }
  }

  /// Adds a mosaic effect to the display. Valid x and y values are between 0 and 3, inclusive.
  pub fn set_mosaic(&mut self, x: u32, y: u32) {
    assert!(x <= 3);
    assert!(y <= 3);
    unsafe { self.state.cdisplay.setMosaic.unwrap()(x, y) }
  }

  /// Flips the display on the x axis iff `flip_x` is true and on the y axis iff `flip_y` is true.
  pub fn set_flipped(&mut self, flip_x: bool, flip_y: bool) {
    unsafe { self.state.cdisplay.setFlipped.unwrap()(flip_x as i32, flip_y as i32) }
  }

  /// Sets the nominal refresh rate in frames per second.
  ///
  /// Default is 20 fps, the maximum rate supported by the hardware for full-frame updates. Note
  /// that the simulator may have a different default refresh rate.
  pub fn set_refresh_rate(&mut self, rate: f32) {
    unsafe { self.state.cdisplay.setRefreshRate.unwrap()(rate) }
  }

  /// Sets the display scale factor. Valid values for scale are 1, 2, 4, and 8.
  ///
  /// The top-left corner of the frame buffer is scaled up to fill the display;
  /// e.g. if the scale is set to 4, the pixels in rectangle [0,100] x [0,60] are drawn on the
  /// screen as 4 x 4 squares.
  pub fn set_scale(&mut self, scale: u32) {
    assert!(scale == 1 || scale == 2 || scale == 4 || scale == 8);
    unsafe { self.state.cdisplay.setScale.unwrap()(scale) }
  }

  /// Offsets the display by the given amount.
  ///
  /// Areas outside of the displayed area are filled with the current background color.
  pub fn set_offset(&mut self, dx: i32, dy: i32) {
    unsafe { self.state.cdisplay.setOffset.unwrap()(dx, dy) }
  }
}
