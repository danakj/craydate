extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let playdate_sdk = env::var("PLAYDATE_SDK_PATH").expect("missing PLAYDATE_SDK_PATH envvar");
    let c_api = PathBuf::from(playdate_sdk).join("C_API");
    let setup_c = c_api
        .join("buildsupport")
        .join("setup.c");

    let clang = env::var("LIBCLANG_PATH").expect("missing LIBCLANG_PATH envvar");

    env::set_var("CFLAGS", "-DTARGET_PLAYDATE=1 -DTARGET_EXTENSION=1");
    env::set_var("CC", PathBuf::from(clang).join("clang"));

    cc::Build::new()
        .file(setup_c)
        .include(c_api)
        .compile("pdsetup");
}