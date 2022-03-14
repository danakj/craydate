use alloc::boxed::Box;
use core::cell::Cell;
use core::ptr::NonNull;

use crate::ctypes::*;
use crate::executor::Executor;

#[non_exhaustive]
#[derive(Debug)]
pub struct CApiState {
  pub capi: &'static CApi,
  pub csystem: &'static CSystem,
  pub cgraphics: &'static CGraphics,
  pub executor: NonNull<Executor>,

  pub frame_number: Cell<u64>,
}
impl CApiState {
  pub fn new(capi: &'static CApi) -> CApiState {
    CApiState {
      cgraphics: unsafe { &*capi.graphics },
      csystem: unsafe { &*capi.system },
      capi,
      executor: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Executor::new()))) },
      frame_number: Cell::new(0),
    }
  }
}
