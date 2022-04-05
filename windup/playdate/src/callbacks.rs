use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ffi::c_void;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::executor::Executor;
use crate::system_event::SystemEvent;

static mut CURRENT_CALLBACK: CallbackArguments = CallbackArguments::None;

#[derive(Debug)]
pub(crate) enum CallbackKey {
  SoundSourceCompletion(*mut CSoundSource),
  MenuItem(*mut c_void),
}
#[derive(Debug)]
enum CallbackArguments {
  None,
  SoundSourceCompletion(*mut CSoundSource),
  MenuItem(*mut c_void),
}
impl CallbackArguments {
  fn is_none(&self) -> bool {
    match self {
      CallbackArguments::None => true,
      _ => false,
    }
  }
}

#[must_use]
pub struct RegisteredCallback {
  cb_type: Option<CallbackKey>,
  weak_removed: Weak<RefCell<Vec<CallbackKey>>>,
}
impl Drop for RegisteredCallback {
  fn drop(&mut self) {
    if let Some(removed) = self.weak_removed.upgrade() {
      removed.borrow_mut().push(self.cb_type.take().unwrap())
    }
  }
}

pub struct Callbacks<T> {
  sound_source_completion_callbacks: BTreeMap<*mut CSoundSource, Box<dyn Fn(T)>>,
  menu_item_callbacks: BTreeMap<*mut c_void, Box<dyn Fn(T)>>,
  removed: Rc<RefCell<Vec<CallbackKey>>>,
}
impl<T> Callbacks<T> {
  pub fn new() -> Self {
    Callbacks {
      sound_source_completion_callbacks: BTreeMap::new(),
      menu_item_callbacks: BTreeMap::new(),
      removed: Rc::new(RefCell::new(Vec::new())),
    }
  }

  fn gc(&mut self) {
    for r in core::mem::take(&mut self.removed).borrow().iter() {
      match r {
        CallbackKey::SoundSourceCompletion(key) => {
          self.sound_source_completion_callbacks.remove(key)
        }
        CallbackKey::MenuItem(key) => self.menu_item_callbacks.remove(key),
      };
    }
  }

  pub fn run(&mut self, t: T) -> bool {
    self.gc();

    match unsafe { &CURRENT_CALLBACK } {
      CallbackArguments::None => false,
      CallbackArguments::SoundSourceCompletion(source_ptr) => {
        let cb = self.sound_source_completion_callbacks.get(source_ptr);
        cb.and_then(|f| Some(f(t))).is_some()
      }
      CallbackArguments::MenuItem(menu_item_ptr) => {
        let cb = self.menu_item_callbacks.get(menu_item_ptr);
        cb.and_then(|f| Some(f(t))).is_some()
      }
    }
  }
}

impl<T> Callbacks<T> {
  pub(crate) fn add_sound_source_completion(
    &mut self,
    key: *mut CSoundSource,
    cb: impl Fn(T) + 'static,
  ) -> (extern "C" fn(*mut CSoundSource), RegisteredCallback) {
    self.sound_source_completion_callbacks.insert(key, Box::new(cb));
    (
      CCallbacks::on_sound_source_completion_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::SoundSourceCompletion(key)),
        weak_removed: Rc::downgrade(&self.removed),
      },
    )
  }

  pub(crate) fn add_menu_item(
    &mut self,
    key: *mut c_void,
    cb: impl Fn(T) + 'static,
  ) -> (extern "C" fn(*mut c_void), RegisteredCallback) {
    self.menu_item_callbacks.insert(key, Box::new(cb));
    (
      CCallbacks::on_menu_item_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::MenuItem(key)),
        weak_removed: Rc::downgrade(&self.removed),
      },
    )
  }
}

struct CCallbacks;
impl CCallbacks {
  fn run_callback(callback_args: CallbackArguments) {
    assert!(unsafe { CURRENT_CALLBACK.is_none() });
    unsafe { CURRENT_CALLBACK = callback_args };
    CApiState::get().add_system_event(SystemEvent::Callback);
    Executor::wake_system_wakers(CApiState::get().executor.as_ptr());
    unsafe { CURRENT_CALLBACK = CallbackArguments::None };
  }

  pub extern "C" fn on_sound_source_completion_callback(key: *mut CSoundSource) {
    Self::run_callback(CallbackArguments::SoundSourceCompletion(key))
  }

  pub extern "C" fn on_menu_item_callback(key: *mut c_void) {
    Self::run_callback(CallbackArguments::MenuItem(key))
  }
}
