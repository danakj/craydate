//! Helpers for the playdate-macro crate. Not meant to be used by human-written code.
extern crate alloc; // `alloc` is fine to use once initialize() has set up the allocator.

use alloc::boxed::Box;
use core::ffi::c_void;

use playdate_sys::playdate_sys as System;
use playdate_sys::PDSystemEvent as SystemEvent;
use playdate_sys::PlaydateAPI as Api;

use crate::*;

extern "C" {
  fn playdate_setup();
  fn playdate_loop();
}

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler1(*mut Api);

// A placeholder to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler2(SystemEvent);

// A placeholder for `u32` to avoid exposing the type/value to playdate's dependent.
#[repr(transparent)]
pub struct EventHandler3(u32);

pub fn initialize(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3) {
  let api = eh1.0;
  let event = eh2.0;
  let _arg = eh3.0;

  // SAFETY: We have made a shared reference to the System. Only refer to the object through
  // the reference hereafter. We can ensure that by never passing a pointer to the `System` or any
  // pointer or reference to the `Api` elsewhere.
  let system: &System = unsafe { &*(*api).system };

  if event == SystemEvent::kEventInit {
    // SAFETY: Do not allocate before the GLOBAL_ALLOCATOR is set up here, or we will crash
    // in the allocator.
    GLOBAL_ALLOCATOR.set_system_ptr(system);

    // We will leak this UpdateCallbackData pointer so it has 'static lifetime.
    //let data_ptr = Box::into_raw(Box::new(Foo::new()));
    unsafe { system.setUpdateCallback.unwrap()(Some(update_callback), system as *const System as *mut c_void) };
  
    unsafe { playdate_setup() };
  }
}

extern "C" fn update_callback(system_ptr: *mut c_void) -> i32 {
  let _system = unsafe { &*(system_ptr as *const System) };
  unsafe { playdate_loop() };
  1
}
