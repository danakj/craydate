#![no_std]
#![deny(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]

use dynpath::dynpath;

pub mod ctypes {
  pub type c_int = i32;
  pub type c_char = u8;
  pub type c_uint = u32;
  pub type c_ulonglong = u64;
  pub type c_void = core::ffi::c_void;
}

#[rustfmt::skip]
#[dynpath("OUT_DIR")]
mod bindings;

pub use bindings::*;
