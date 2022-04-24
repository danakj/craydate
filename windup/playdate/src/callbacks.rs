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

/// The key used for each set of callbacks held in a `Callbacks` collection.
///
/// They key type would need to be passed to the C callback function in order to find the
/// user-provided closure from the key.
#[derive(Debug)]
enum CallbackKey {
  SoundSourceCompletion(usize),
  MenuItem(usize),
  SequenceFinished(usize),
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
  removed: Rc<RefCell<Vec<CallbackKey>>>,
}
impl<T> Callbacks<T> {
  /// Construct a container for callbacks that will be passed `T` when they are run.
  pub fn new() -> Self {
    Callbacks {
      sound_source_completion_callbacks: BTreeMap::new(),
      menu_item_callbacks: BTreeMap::new(),
      sequence_finished_callbacks: BTreeMap::new(),
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
        CallbackKey::SequenceFinished(key) => self.sequence_finished_callbacks.remove(key),
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
    self.sound_source_completion_callbacks.insert(key, Box::new(cb));
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
    self.menu_item_callbacks.insert(key, Box::new(cb));
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
  ) -> (unsafe extern "C" fn(*mut CSoundSequence, *mut c_void), RegisteredCallback) {
    self.sound_source_completion_callbacks.insert(key, Box::new(cb));
    (
      CCallbacks::on_sequence_finished_callback,
      RegisteredCallback {
        cb_type: Some(CallbackKey::SequenceFinished(key)),
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
}

pub enum NoNull {}
pub enum AllowNull {}
pub enum Unconstructed {}
pub enum WithCallacks {}
pub enum Constructed {}

/// A builder pattern to construct a callback that will later be called when `SystemEvent::Callback`
/// fires. Connects a closure to a `Callbacks` object which can later run the closure.
pub struct CallbackBuilder<
  'a,
  T = (),
  F: Fn(T) + 'static = fn(T),
  Rule = AllowNull,
  State = Unconstructed,
> {
  callbacks: Option<&'a mut Callbacks<T>>,
  cb: Option<F>,
  _marker: core::marker::PhantomData<(&'a u8, T, F, Rule, State)>,
}
impl<'a> CallbackBuilder<'a, (), fn(()), AllowNull, Unconstructed> {
  /// A null callback, which is used to specify a callback should not be set, or should be removed.
  pub fn none() -> CallbackBuilder<'a, (), fn(()), AllowNull, Constructed> {
    CallbackBuilder {
      callbacks: None,
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, Unconstructed> {
  /// Attach a `Callbacks` object to this builder, that will hold the closure.
  pub fn with(callbacks: &'a mut Callbacks<T>) -> CallbackBuilder<'a, T, F, Rule, WithCallacks> {
    CallbackBuilder {
      callbacks: Some(callbacks),
      cb: None,
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, WithCallacks> {
  /// Attach a closure to this builder, which will be held in the `Callbacks` object and called via
  /// that same `Callbacks` object.
  pub fn call(self, cb: F) -> CallbackBuilder<'a, T, F, Rule, Constructed> {
    CallbackBuilder {
      callbacks: self.callbacks,
      cb: Some(cb),
      _marker: core::marker::PhantomData,
    }
  }
}
impl<'a, T, F: Fn(T) + 'static, Rule> CallbackBuilder<'a, T, F, Rule, Constructed> {
  pub(crate) fn into_inner(self) -> Option<(&'a mut Callbacks<T>, F)> {
    self.callbacks.zip(self.cb)
  }
}
