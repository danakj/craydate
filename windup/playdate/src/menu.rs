use alloc::string::String;
use core::ffi::c_void;

use crate::callbacks::*;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;

static mut MENU_KEY: *mut c_void = core::ptr::null_mut();

pub struct MenuItem {
  ptr: *mut CMenuItem,
  title: String,
  #[allow(dead_code)]
  callback: RegisteredCallback,  // Holds ownership of the closure.
}

impl MenuItem {
  pub fn new_action<T>(
    title: &str,
    cb: impl Fn(T) + 'static,
    callbacks: &mut Callbacks<T>,
  ) -> MenuItem {
    let key = unsafe {
      MENU_KEY = MENU_KEY.add(1);
      MENU_KEY
    };
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let title = String::from(title); // Allocate a stable title pointer to pass to C.
    let ptr = unsafe {
      CApiState::get().csystem.addMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr,
      title,
      callback: reg,
    }
  }

  pub fn title(&self) -> &str {
    &self.title
  }
  pub fn set_title(&mut self, title: &str) {
    let title = String::from(title);
    unsafe {
      CApiState::get().csystem.setMenuItemTitle.unwrap()(
        self.ptr,
        self.title.to_null_terminated_utf8().as_ptr(),
      )
    }
    self.title = title;
  }
}

impl Drop for MenuItem {
  fn drop(&mut self) {
    unsafe { CApiState::get().csystem.removeMenuItem.unwrap()(self.ptr) };
  }
}
