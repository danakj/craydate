use alloc::rc::Rc;
use core::ptr::NonNull;

use crate::ctypes::*;

/// A `SynthSignal` represents a signal that can be used as a modulator for a `Synth`.
/// 
/// Cloning a `SynthSignal` makes a shallow copy, where operations on the original or on the clone
/// both act on the same underlying signal.
#[derive(Clone)]
pub struct SynthSignal {
  // Non-owning pointer, attached to the lifetime of the `subclass` object.
  ptr: NonNull<CSynthSignalValue>,
  // An opaque trait object which is present just to manage the lifetime of any resources owned by
  // the subclass. Once a SynthSignal subclass is converted to a SynthSignal, its type is lost but
  // it continues to function and this trait object holds the data needed by it.
  _subclass: Rc<dyn SynthSignalSubclass>,
}
impl SynthSignal {
  pub(crate) fn new(ptr: *mut CSynthSignalValue, subclass: Rc<dyn SynthSignalSubclass>) -> Self {
    SynthSignal { ptr: NonNull::new(ptr).unwrap(), _subclass: subclass }
  }

  pub(crate) fn cptr(&self) -> *mut CSynthSignalValue {
    self.ptr.as_ptr()
  }
}

impl core::fmt::Debug for SynthSignal {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SynthSignal").field("ptr", &self.ptr).finish()
  }
}

pub(crate) trait SynthSignalSubclass {}
