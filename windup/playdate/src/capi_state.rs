use alloc::boxed::Box;
use core::ptr::NonNull;

use crate::ctypes::*;
use crate::executor::Executor;

#[non_exhaustive]
pub struct CApiState {
  pub api: &'static CApi,
  pub system: &'static CSystem,
  pub graphics: &'static CGraphics,
  pub executor: NonNull<Executor>,
}
impl CApiState {
  pub fn new(api: &'static CApi) -> CApiState {
    CApiState {
      graphics: unsafe { &*api.graphics },
      system: unsafe { &*api.system },
      api,
      executor: unsafe {
        NonNull::new_unchecked(Box::into_raw(Box::new(Executor::new())))
      },
    }
  }
}
