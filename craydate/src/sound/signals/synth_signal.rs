use alloc::rc::Rc;
use core::ptr::NonNull;

use crate::ctypes::*;

/// A `SynthSignal` represents a signal that can be used as a modulator for a `Synth`.
///
/// There are other types that act as a `SynthSignal`. Any such type would implement
/// `AsRef<SynthSignal>` and `AsMut<SynthSignal>`. They also have `as_signal()` and
/// `as_signal_mut()` methods, through the `AsSynthSignal` trait, to access the `SynthSignal`
/// methods more easily.
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
    SynthSignal {
      ptr: NonNull::new(ptr).unwrap(),
      _subclass: subclass,
    }
  }

  // Note: There is no visible state on SynthSignal, as seen by the lack of methods on this type.
  // We give a mutable pointer to it to C when setting a SynthSignal. Since there's no mutable state
  // we don't need to worry about converting from a const pointer to mut.
  pub(crate) fn cptr(&self) -> *const CSynthSignalValue {
    self.ptr.as_ptr()
  }
}

impl core::fmt::Debug for SynthSignal {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SynthSignal").field("ptr", &self.ptr).finish()
  }
}

pub(crate) trait SynthSignalSubclass {}

/// Provides explicit access to a type's `SynthSignal` methods when it can act as a `SynthSignal`.
pub trait AsSynthSignal: AsRef<SynthSignal> + AsMut<SynthSignal> {
  fn as_signal(&self) -> &SynthSignal {
    self.as_ref()
  }
  fn as_signal_mut(&mut self) -> &mut SynthSignal {
    self.as_mut()
  }
}
impl<T> AsSynthSignal for T where T: AsRef<SynthSignal> + AsMut<SynthSignal> {}
