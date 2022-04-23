use alloc::vec::Vec;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::callbacks::*;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::null_terminated::ToNullTerminatedString;

/// A callback builder for a closure to be called on menu events.
pub type MenuCallback<'a, T, F, S> = crate::callbacks::CallbackBuilder<'a, T, F, NoNull, S>;

static mut MENU_KEY: usize = 0;
/// Makes a unique id to pass as a "userdata" key to determine which callback is being called.
fn make_callback_key() -> usize {
  unsafe {
    MENU_KEY += 1;
    MENU_KEY
  }
}

pub enum Action {}
pub enum Checkmark {}
pub enum Options {}
pub enum AnyType {}

/// A system menu item. The game can specify up to 3 custom menu items in the system menu.
pub struct MenuItem<Type = AnyType> {
  ptr: NonNull<CMenuItem>,
  _callback: RegisteredCallback, // Holds ownership of the closure.
  _marker: PhantomData<Type>,
}

impl MenuItem {
  /// Construct a new action menu item and add it to the system menu as long as the MenuItem stays
  /// alive.
  ///
  /// The callback will be registered as a system event. If the action menu item is chosen, the menu
  /// will be closed and the the application will be notified to run the callback via a
  /// `SystemEvent::Callback` event. When that occurs, the application's `Callbacks` object which
  /// was used to construct the `completion_callback` can be `run()` to execute the closure bound in
  /// the `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// menu.new_action("action", MenuCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("action happened");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.runs();
  ///   }
  /// }
  /// ```
  pub fn new_action<'a, T, F: Fn(T) + 'static>(
    title: &str,
    callback: MenuCallback<'a, T, F, Constructed>,
  ) -> MenuItem<Action> {
    let key = make_callback_key();
    let (callbacks, cb) = callback.into_inner().unwrap();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let ptr = unsafe {
      Self::fns().addMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr: NonNull::new(ptr).unwrap(),
      _callback: reg,
      _marker: PhantomData,
    }
  }

  /// Construct a new checkmark menu item and add it to the system menu as long as the MenuItem
  /// stays alive.
  ///
  /// The callback will be registered as a system event.
  ///
  /// If the action menu item is chosen, the value will be changed, and when menu is later closed
  /// the application will be notified to run the callback via a `SystemEvent::Callback` event. When
  /// that occurs, the application's `Callbacks` object which was used to construct the
  /// `completion_callback` can be `run()` to execute the closure bound in the
  /// `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// menu.new_checkmark("check", true, MenuCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("checkmark changed");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.runs();
  ///   }
  /// }
  /// ```
  pub fn new_checkmark<'a, T, F: Fn(T) + 'static>(
    title: &str,
    intially_checked: bool,
    callback: MenuCallback<'a, T, F, Constructed>,
  ) -> MenuItem<Checkmark> {
    let key = make_callback_key();
    let (callbacks, cb) = callback.into_inner().unwrap();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let ptr = unsafe {
      Self::fns().addCheckmarkMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        intially_checked as i32,
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr: NonNull::new(ptr).unwrap(),
      _callback: reg,
      _marker: PhantomData,
    }
  }

  /// Construct a new options menu item and add it to the system menu as long as the MenuItem stays
  /// alive.
  ///
  /// The callback will be registered as a system event.
  ///
  /// If the action menu item is chosen, the value will be changed, and when menu is later closed
  /// the application will be notified to run the callback via a `SystemEvent::Callback` event. When
  /// that occurs, the application's `Callbacks` object which was used to construct the
  /// `completion_callback` can be `run()` to execute the closure bound in the
  /// `completion_callback`.
  ///
  /// # Example
  /// ```
  /// let callbacks: Callbacks<i32> = Callbacks::new();
  /// // Register a closure as a callback.
  /// menu.new_options("values", options, MenuCallback::with(&mut callbacks).call(|i: i32| {
  ///   println("value changed");
  /// }));
  /// match system_event_watcher.next() {
  ///   SystemEvent::Callback => {
  ///     // Run the closure registered above.
  ///     callbacks.runs();
  ///   }
  /// }
  /// ```
  pub fn new_options<'a, T, F: Fn(T) + 'static>(
    title: &str,
    options: impl IntoIterator<Item = &'a str>,
    callback: MenuCallback<'a, T, F, Constructed>,
  ) -> MenuItem<Options> {
    let key = make_callback_key();
    let (callbacks, cb) = callback.into_inner().unwrap();
    let (func, reg) = callbacks.add_menu_item(key, cb);
    let options_null_terminated: Vec<_> =
      options.into_iter().map(|o| o.to_null_terminated_utf8()).collect();
    let options_pointers: Vec<_> = options_null_terminated.iter().map(|o| o.as_ptr()).collect();
    let ptr = unsafe {
      Self::fns().addOptionsMenuItem.unwrap()(
        title.to_null_terminated_utf8().as_ptr(),
        options_pointers.as_ptr() as *mut *const u8,
        options_pointers.len() as i32,
        Some(func),
        key as *mut c_void,
      )
    };
    MenuItem {
      ptr: NonNull::new(ptr).unwrap(),
      _callback: reg,
      _marker: PhantomData,
    }
  }
}

impl<T> MenuItem<T> {
  /// Get the menu item's title.
  pub fn title(&self) -> &str {
    let ptr = unsafe { Self::fns().getMenuItemTitle.unwrap()(self.cptr()) };
    // SAFETY: Strings returned from playdate are utf8 and null-terminated.
    unsafe { crate::null_terminated::parse_null_terminated_utf8(ptr).unwrap() }
  }
  /// Set the menu item's title.
  pub fn set_title(&mut self, title: &str) {
    unsafe {
      Self::fns().setMenuItemTitle.unwrap()(self.cptr(), title.to_null_terminated_utf8().as_ptr())
    }
  }

  pub(crate) fn cptr(&self) -> *mut CMenuItem {
    self.ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static playdate_sys::playdate_sys {
    CApiState::get().csystem
  }
}

impl MenuItem<Checkmark> {
  /// Returns if the checkmark menu item was checked when the menu was closed.
  pub fn checked(&self) -> bool {
    unsafe { Self::fns().getMenuItemValue.unwrap()(self.cptr()) != 0 }
  }
  /// Sets if the checkmark menu item should be checked when the menu is next opened.
  pub fn set_checked(&self, checked: bool) {
    unsafe { Self::fns().setMenuItemValue.unwrap()(self.cptr(), checked as i32) }
  }
}

impl MenuItem<Options> {
  /// Returns the index of the option that was selected when the menu was closed.
  pub fn value(&self) -> i32 {
    unsafe { Self::fns().getMenuItemValue.unwrap()(self.cptr()) }
  }
  /// Sets the index of the option to be selected when the menu is next opened.
  pub fn set_value(&self, value: i32) {
    unsafe { Self::fns().setMenuItemValue.unwrap()(self.cptr(), value) }
  }
}

impl<Type> Drop for MenuItem<Type> {
  fn drop(&mut self) {
    unsafe { CApiState::get().csystem.removeMenuItem.unwrap()(self.cptr()) };
  }
}
