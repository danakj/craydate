use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ffi::c_void;

use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::executor::Executor;
use crate::sound::headphone_state::HeadphoneState;
use crate::system_event::SystemEvent;

static mut CURRENT_CALLBACK: CallbackArguments = CallbackArguments::None;

/// The key used for each set of callbacks held in a `Callbacks` collection.
///
/// They key type would need to be passed to the C callback function in order to find the
/// user-provided closure from the key.
#[derive(Debug)]
enum CallbackKey {
  SoundSourceCompletion(usize),
  MenuItem(usize),
  SequenceFinished(usize),
  HeadphoneChanged,
}

/// The arguments given to the C callback function for each type of function. These are used to find
/// the user-provided closure.
///
/// The enum functions to indicate, in `CURRENT_CALLBACK`, which callback is currently being
/// executed, or `None`.
#[derive(Debug)]
enum CallbackArguments {
  /// Indicates that no callback is active.
  None,
  SoundSourceCompletion(usize),
  MenuItem(usize),
  SequenceFinished(usize),
  HeadphoneChanged(HeadphoneState),
}
impl CallbackArguments {
  fn is_none(&self) -> bool {
    match self {
      CallbackArguments::None => true,
      _ => false,
    }
  }
}

/// Holds ownership of the closure given when registering a system callback. Dropping this type
/// would prevent the closure from ever being called. Typically held as a field as long as a
/// callback is registered.
#[must_use]
#[derive(Debug)]
pub(crate) struct RegisteredCallback {
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

/// Provides an API to run a closure tied to a callback when the `SystemEventWatcher` reports a
/// callback is ready to be run via `SystemEvent::Callback`. This type uses its type argument `T` to
/// define the values that the caller will pass along to the closure when running it.
pub struct Callbacks<T> {
  sound_source_completion_callbacks: BTreeMap<usize, Box<dyn Fn(T)>>,
  menu_item_callbacks: BTreeMap<usize, Box<dyn Fn(T)>>,
  sequence_finished_callbacks: BTreeMap<usize, Box<dyn Fn(T)>>,
  headphone_changed_callback: Option<Box<dyn Fn(HeadphoneState, T)>>,
  removed: Rc<RefCell<Vec<CallbackKey>>>,
}
impl<T> Callbacks<T> {
  /// Construct a container for callbacks that will be passed `T` when they are run.
  pub fn new() -> Self {
    Callbacks {
      sound_source_completion_callbacks: BTreeMap::new(),
      menu_item_callbacks: BTreeMap::new(),
      sequence_finished_callbacks: BTreeMap::new(),
      headphone_changed_callback: None,
      removed: Rc::new(RefCell::new(Vec::new())),
    }
  }

  fn gc(&mut self) {
    for r in core::mem::take(&mut self.removed).borrow().iter() {
      match r {
        CallbackKey::SoundSourceCompletion(key) => {
          self.sound_source_completion_callbacks.remove(key);
        }
        CallbackKey::MenuItem(key) => {
          self.menu_item_callbacks.remove(key);
        }
        CallbackKey::SequenceFinished(key) => {
          self.sequence_finished_callbacks.remove(key);
        }
        CallbackKey::HeadphoneChanged => {
          self.headphone_changed_callback = None;
        }
      };
    }
  }

  /// Attempt to run a callback, passing along a `T`.
  ///
  /// This should be called in response to a `SystemEvent::Callback` event occuring, which indicates
  /// there is a callback available to be run.
  ///
  /// Returns true if the callback was found in this `Callbacks` collection and run, otherwise
  /// returns false. A false return would mean the callback is in a different `Callbacks` collection
  /// (or the callback was removed via dropping `RegisteredCallback` internally incorrectly).
  pub fn run(&mut self, t: T) -> bool {
    self.gc();

    match unsafe { &CURRENT_CALLBACK } {
      CallbackArguments::None => false,
      CallbackArguments::SoundSourceCompletion(key) => {
        let cb = self.sound_source_completion_callbacks.get(key);
        cb.and_then(|f| Some(f(t))).is_some()
      }
      CallbackArguments::MenuItem(key) => {
        let cb = self.menu_item_callbacks.get(key);
        cb.and_then(|f| Some(f(t))).is_some()
      }
      CallbackArguments::SequenceFinished(key) => {
        let cb = self.sequence_finished_callbacks.get(key);
        cb.and_then(|f| Some(f(t))).is_some()
      }
      CallbackArguments::HeadphoneChanged(state) => {
        let cb = self.headphone_changed_callback.as_ref();
        cb.and_then(|f| Some(f(*state, t))).is_some()
      }
    }
  }
}

impl<T> Callbacks<T> {
  #[must_use]
  pub(crate) fn add_sound_source_completion(
    &mut self,
    key: usize,
    cb: impl Fn(T) + 'static,
  ) -> (unsafe extern "C" fn(*mut CSoundSource), RegisteredCallback) {
    let r = self.sound_source_completion_callbacks.insert(key, Box::new(cb));
    assert!(r.is_none());
    (
      CCallbacks::on_sound_source_completion_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::SoundSourceCompletion(key)),
        weak_removed: Rc::downgrade(&self.removed),
      },
    )
  }

  #[must_use]
  pub(crate) fn add_menu_item(
    &mut self,
    key: usize,
    cb: impl Fn(T) + 'static,
  ) -> (unsafe extern "C" fn(*mut c_void), RegisteredCallback) {
    let r = self.menu_item_callbacks.insert(key, Box::new(cb));
    assert!(r.is_none());
    (
      CCallbacks::on_menu_item_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::MenuItem(key)),
        weak_removed: Rc::downgrade(&self.removed),
      },
    )
  }

  #[must_use]
  pub(crate) fn add_sequence_finished(
    &mut self,
    key: usize,
    cb: impl Fn(T) + 'static,
  ) -> (
    unsafe extern "C" fn(*mut CSoundSequence, *mut c_void),
    RegisteredCallback,
  ) {
    let r = self.sound_source_completion_callbacks.insert(key, Box::new(cb));
    assert!(r.is_none());
    (
      CCallbacks::on_sequence_finished_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::SequenceFinished(key)),
        weak_removed: Rc::downgrade(&self.removed),
      },
    )
  }

  #[must_use]
  pub(crate) fn add_headphone_change(
    &mut self,
    cb: impl Fn(HeadphoneState, T) + 'static,
  ) -> (unsafe extern "C" fn(i32, i32), RegisteredCallback) {
    assert!(self.headphone_changed_callback.is_none());
    self.headphone_changed_callback = Some(Box::new(cb));
    (
      CCallbacks::on_headphone_change_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::HeadphoneChanged),
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
    // Waking the executors should cause them to poll() and receive back a `SystemEvent::Callback`,
    // as set on the line above. They would run the callback via `Callbacks` then eventually yield
    // back to us here.
    Executor::wake_system_wakers(CApiState::get().executor);
    unsafe { CURRENT_CALLBACK = CallbackArguments::None };
  }

  pub extern "C" fn on_sound_source_completion_callback(key: *mut CSoundSource) {
    Self::run_callback(CallbackArguments::SoundSourceCompletion(key as usize))
  }

  pub extern "C" fn on_menu_item_callback(key: *mut c_void) {
    Self::run_callback(CallbackArguments::MenuItem(key as usize))
  }

  pub extern "C" fn on_sequence_finished_callback(seq: *mut CSoundSequence, _data: *mut c_void) {
    Self::run_callback(CallbackArguments::SequenceFinished(seq as usize))
  }

  pub extern "C" fn on_headphone_change_callback(headphones: i32, mic: i32) {
    Self::run_callback(CallbackArguments::HeadphoneChanged(HeadphoneState::new(
      headphones != 0,
      mic != 0,
    )))
  }
}
