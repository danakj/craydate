use core::marker::PhantomData;
use core::ptr::NonNull;

use super::super::signals::control::{Control, ControlRef};
use crate::ctypes::*;

// A `Control` which is owned by a `SequenceTrack`.
pub struct SequenceTrackControl<'a> {
  cref: ControlRef,
  _marker: PhantomData<&'a Control>,
}
impl SequenceTrackControl<'_> {
  pub(crate) fn from_ptr(ptr: NonNull<CControlSignal>) -> Self {
    SequenceTrackControl {
      cref: ControlRef::from_ptr(ptr),
      _marker: PhantomData,
    }
  }
}

impl core::ops::Deref for SequenceTrackControl<'_> {
  type Target = ControlRef;

  fn deref(&self) -> &Self::Target {
    &self.cref
  }
}
impl core::ops::DerefMut for SequenceTrackControl<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.cref
  }
}
impl AsRef<ControlRef> for SequenceTrackControl<'_> {
  fn as_ref(&self) -> &ControlRef {
    self
  }
}
impl AsMut<ControlRef> for SequenceTrackControl<'_> {
  fn as_mut(&mut self) -> &mut ControlRef {
    self
  }
}
