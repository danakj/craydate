//! To regenerate the bindings module: `cargo run --features=generate`

#[cfg(not(feature = "generate"))]
fn main() {
  println!(
    "ERROR: to generate bindings, build and run with the 'generate' feature: \n  \
    cargo run --features=generate"
  );
}

#[cfg(feature = "generate")]
use bindgen;

#[cfg(feature = "generate")]
fn main() -> std::result::Result<(), std::io::Error> {
  use std::env;
  use std::path::PathBuf;

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
    .clang_arg(format!("-I{}", c_api.to_str().unwrap()))
    .clang_arg("-DTARGET_EXTENSION=1")
    .clang_arg("-v")
    .allowlist_function("eventHandler")
    .allowlist_type("PlaydateAPI")
    .allowlist_type("PDSystemEvent")
    .allowlist_type("LCDPattern")
    .allowlist_var("SEEK_SET")
    .allowlist_var("SEEK_CUR")
    .allowlist_var("SEEK_END")
    .allowlist_var("LCD_COLUMNS")
    .allowlist_var("LCD_ROWS")
    .allowlist_var("LCD_ROWSIZE")
    .newtype_enum("LCDSolidColor")
    .newtype_enum("PDSystemEvent")
    .newtype_enum("LCDBitmapDrawMode")
    .newtype_enum("LCDBitmapFlip")
    .newtype_enum("LCDPolygonFillRule")
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

  let mut bindgen_out = Vec::new();
  bindings.write(Box::new(&mut bindgen_out)).expect("Couldn't write bindings!");

  const HEADER: &str = "// To regenerate this bindings module: `cargo run --features=generate`\n\
    \n\
    #![allow(deref_nullptr)]\n\n";

  // Write the bindings to the src/bindings.rs file.
  let mut file_out = Vec::new();
  file_out.extend(HEADER.as_bytes());
  file_out.extend(bindgen_out.into_iter());
  let out_path = PathBuf::from("src").join("bindings.rs");
  std::fs::write(&out_path, file_out)?;
  println!(
    "Successfully generated bindings in {}!",
    out_path.to_str().unwrap()
  );
  Ok(())
}
