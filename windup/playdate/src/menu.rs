use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::marker::PhantomData;

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

pub enum Action {}
pub enum Checkmark {}
pub enum Options {}
pub enum AnyType {}

/// A system menu item. The game can specify up to 3 custom menu items in the system menu.
pub struct MenuItem<Type = AnyType> {
  ptr: *mut CMenuItem,
  _callback: RegisteredCallback, // Holds ownership of the closure.
  _marker: PhantomData<Type>,
}

impl MenuItem {
  /// Construct a new action menu item and add it to the system menu as long as the MenuItem stays
  /// alive.
  ///
  /// If the action menu item is chosen, the menu will be closed and the given callback `cb` will be
  /// available to run. A `SystemEvent::Callback` event will fire to indicate this.
  pub fn new_action<T>(
    title: &str,
    callbacks: &mut Callbacks<T>,
    cb: impl Fn(T) + 'static,
  ) -> MenuItem<Action> {
    let key = make_key();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let ptr = unsafe {
      CApiState::get().csystem.addMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr,
      _callback: reg,
      _marker: PhantomData,
    }
  }

  /// Construct a new checkmark menu item and add it to the system menu as long as the MenuItem
  /// stays alive.
  ///
  /// If the action menu item is chosen, the value will be changed, and when menu is later closed
  /// the given callback `cb` will be available to run. A `SystemEvent::Callback` event will fire to
  /// indicate this.
  pub fn new_checkmark<T>(
    title: &str,
    intially_checked: bool,
    callbacks: &mut Callbacks<T>,
    cb: impl Fn(T) + 'static,
  ) -> MenuItem<Checkmark> {
    let key = make_key();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let ptr = unsafe {
      CApiState::get().csystem.addCheckmarkMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        intially_checked as i32,
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr,
      _callback: reg,
      _marker: PhantomData,
    }
  }

  /// Construct a new options menu item and add it to the system menu as long as the MenuItem stays
  /// alive.
  ///
  /// If the action menu item is chosen, the value will be changed, and when menu is later closed
  /// the given callback `cb` will be available to run. A `SystemEvent::Callback` event will fire to
  /// indicate this.
  pub fn new_options<'a, T>(
    title: &str,
    options: impl IntoIterator<Item = &'a str>,
    callbacks: &mut Callbacks<T>,
    cb: impl Fn(T) + 'static,
  ) -> MenuItem<Options> {
    let key = make_key();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let options_null_terminated: Vec<_> =
      options.into_iter().map(|o| o.to_null_terminated_utf8()).collect();
    let options_pointers: Vec<_> = options_null_terminated.iter().map(|o| o.as_ptr()).collect();
    let ptr = unsafe {
      CApiState::get().csystem.addOptionsMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        options_pointers.as_ptr() as *mut *const u8,
        options_pointers.len() as i32,
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr,
      _callback: reg,
      _marker: PhantomData,
    }
  }
}

impl<T> MenuItem<T> {
  /// Get the menu item's title.
  pub fn title(&self) -> &str {
    unsafe {
      let ptr = CApiState::get().csystem.getMenuItemTitle.unwrap()(self.ptr);
      crate::null_terminated::parse_null_terminated_utf8(ptr).unwrap()
    }
  }
  /// Set the menu item's title.
  pub fn set_title(&mut self, title: &str) {
    let title = String::from(title);
    unsafe {
      CApiState::get().csystem.setMenuItemTitle.unwrap()(
        self.ptr,
        title.to_null_terminated_utf8().as_ptr(),
      )
    }
  }
}

impl MenuItem<Checkmark> {
  pub fn checked(&self) -> bool {
    unsafe { CApiState::get().csystem.getMenuItemValue.unwrap()(self.ptr) != 0 }
  }
  pub fn set_checked(&self, checked: bool) {
    unsafe { CApiState::get().csystem.setMenuItemValue.unwrap()(self.ptr, checked as i32) }
  }
}

impl MenuItem<Options> {
  pub fn value(&self) -> i32 {
    unsafe { CApiState::get().csystem.getMenuItemValue.unwrap()(self.ptr) }
  }
  pub fn set_value(&self, value: i32) {
    unsafe { CApiState::get().csystem.setMenuItemValue.unwrap()(self.ptr, value) }
  }
}

impl<Type> Drop for MenuItem<Type> {
  fn drop(&mut self) {
    unsafe { CApiState::get().csystem.removeMenuItem.unwrap()(self.ptr) };
  }
}
