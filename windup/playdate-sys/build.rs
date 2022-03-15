extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
  // Tell cargo to invalidate the built crate whenever the wrapper changes
  println!("cargo:rerun-if-changed=wrapper.h");

  let playdate_sdk =
    env::var("PLAYDATE_SDK_PATH").expect("Set PLAYDATE_SDK_PATH to the correct path");
  let c_api = PathBuf::from(playdate_sdk).join("C_API");

  // The bindgen::Builder is the main entry point
  // to bindgen, and lets you build up options for
  // the resulting bindings.
  let builder = bindgen::Builder::default();
  // It's not clear if we use TARGET_SIMULATOR or PLAYDATE_SIMULATOR. Half the examples
  // use one and half use the other.
  #[cfg(not(all(target_arch = "arm", target_os = "none")))]
  let builder = builder.clang_arg("-DTARGET_SIMULATOR=1");
  #[cfg(not(all(target_arch = "arm", target_os = "none")))]
  let builder = builder.clang_arg("-DPLAYDATE_SIMULATOR=1");
  #[cfg(all(target_arch = "arm", target_os = "none"))]
  let builder = builder.clang_arg("-DTARGET_PLAYDATE=1");

  let bindings = builder
    .use_core()
    .ctypes_prefix("super::ctypes")
    .clang_arg("--target=x86_64-pc-windows-msvc")
    .clang_arg(format!("-I{}", c_api.to_str().unwrap()))
    .clang_arg("-DTARGET_EXTENSION=1")
    .clang_arg("-v")
    .allowlist_function("eventHandler")
    .allowlist_type("PlaydateAPI")
    .allowlist_type("PDSystemEvent")
    .allowlist_type("LCDPattern")
    .newtype_enum("LCDSolidColor")
    .newtype_enum("PDSystemEvent")
    .newtype_enum("LCDBitmapDrawMode")
    .newtype_enum("LCDBitmapFlip")
    .newtype_enum("LCDLineCapStyle")
    .newtype_enum("PDStringEncoding")
    .newtype_enum("LCDPolygonFillRule")
    .newtype_enum("PDLanguage")
    .bitfield_enum("PDPeripherals")
    .newtype_enum("SpriteCollisionResponseType")
    .newtype_enum("SoundFormat")
    .newtype_enum("LFOType")
    .newtype_enum("SoundWaveform")
    .newtype_enum("TwoPoleFilterType")
    .bitfield_enum("PDButtons")
    .bitfield_enum("FileOptions")
    // TODO: maybe do something with these if needed
    .opaque_type("playdate_json")
    .opaque_type("playdate_lua")
    // The input header we would like to generate
    // bindings for.
    .header("wrapper.h")
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}
