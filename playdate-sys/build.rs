extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    // println!("cargo:rustc-link-lib=bz2");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let playdate_sdk = env::var("PLAYDATE_SDK_PATH").unwrap();
    let c_api = PathBuf::from(playdate_sdk).join("C_API");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", c_api.to_str().unwrap()))
        .clang_arg("-DTARGET_EXTENSION")
        .clang_arg("-v")
        .allowlist_function("eventHandler")
        .allowlist_type("PlaydateAPI")
        .allowlist_type("PDSystemEvent")
        .newtype_enum("PDSystemEvent")
        .newtype_enum("LCDBitmapDrawMode")
        .newtype_enum("LCDBitmapFlip")
        .newtype_enum("LCDLineCapStyle")
        .newtype_enum("PDStringEncoding")
        .newtype_enum("LCDPolygonFillRule")
        .newtype_enum("LCDSolidColor")
        .newtype_enum("PDLanguage")
        .newtype_enum("PDPeripherals")
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
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}