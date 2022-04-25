/// The state of the headphone jack.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeadphoneState {
  HeadphoneNotConnected,
  HeadphoneConnected { has_microphone: bool },
}
impl HeadphoneState {
  pub(crate) fn new(headphones: bool, mic: bool) -> Self {
    if !headphones {
      HeadphoneState::HeadphoneNotConnected
    } else {
      HeadphoneState::HeadphoneConnected {
        has_microphone: mic,
      }
    }
  }
}
