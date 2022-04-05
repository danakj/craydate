use alloc::string::String;
use core::ffi::c_void;

use crate::callbacks::*;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;

static mut MENU_KEY: *mut c_void = core::ptr::null_mut();
fn make_key() -> *mut c_void {
  unsafe {
    MENU_KEY = MENU_KEY.add(1);
    MENU_KEY
  }
}

/// A system menu item. The game can specify up to 3 custom menu items in the system menu.
pub struct MenuItem {
  ptr: *mut CMenuItem,
  title: String,
  #[allow(dead_code)]
  callback: RegisteredCallback, // Holds ownership of the closure.
}

impl MenuItem {
  /// Construct a new action menu item and add it to the system menu as long as the MenuItem stays
  /// alive.
  /// 
  /// If the action menu item is chosen, the menu will be closed and the given callback `cb` will be
  /// available to run. A `SystemEvent::Callback` event will fire to indicate this.
  pub fn new_action<T>(
    title: &str,
    cb: impl Fn(T) + 'static,
    callbacks: &mut Callbacks<T>,
  ) -> MenuItem {
    let key = make_key();
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

  /// Get the menu item's title.
  pub fn title(&self) -> &str {
    &self.title
  }
  /// Set the menu item's title.
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
