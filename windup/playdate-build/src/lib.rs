/// Consts used to configure behaviour that may be controlled by cfgs.
mod consts;
/// Errors that can be returned from the crate.
mod error;

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX};
use std::path::PathBuf;
use std::process::Command;

pub use error::{PlaydateBuildError, Result};

fn pdx_source_dir() -> PathBuf {
    let dir = std::env::var("OUT_DIR").expect("OUT_DIR envionment variable is not set");
    PathBuf::from(dir).join("pdx_source")
}

fn pdx_out_dir() -> PathBuf {
    let dir = std::env::var("OUT_DIR").expect("OUT_DIR envionment variable is not set");
    PathBuf::from(dir).join("pdx_out")
}

fn pdx_name() -> String {
    std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME envionment variable is not set")
}

pub fn export_vars() {
    println!(
        "cargo:rustc-env={}={}",
        "PDX_SOURCE_DIR",
        pdx_source_dir().to_string_lossy()
    );
    println!(
        "cargo:rustc-env={}={}",
        "PDX_OUT_DIR",
        pdx_out_dir().to_string_lossy()
    );
    println!("cargo:rustc-env={}={}", "PDX_NAME", pdx_name());
}

pub fn build_pdx(pdx_source_dir: &str, pdx_out_dir: &str, pdx_name: &str) -> Result<String> {
    let sdk_path = std::env::var("PLAYDATE_SDK_PATH")
        .expect("PLAYDATE_SDK_PATH environment variable is not set");

    let pdx_source_dir = PathBuf::from(pdx_source_dir);
    let pdx_out_dir = PathBuf::from(pdx_out_dir);

    std::fs::create_dir_all(&pdx_source_dir)?;
    std::fs::create_dir_all(&pdx_out_dir)?;

    // Touch the source pdx.bin file, which is empty for the simulator target.
    std::fs::write(pdx_source_dir.join("pdex.bin"), "")?;

    // Copy the library into the source dir for the compiler.
    let lib_name = format!("{}{}{}", DLL_PREFIX, pdx_name.replace('-', "_"), DLL_SUFFIX);
    let pdex_lib_name = format!("{}{}{}", DLL_PREFIX, "pdex", DLL_SUFFIX);
    let lib_path = &pdx_out_dir;
    let lib_path = lib_path.parent().unwrap(); // Cargo OUT_DIR.
    let lib_path = lib_path.parent().unwrap(); // Cargo crate dir.
    let lib_path = lib_path.parent().unwrap(); // Cargo build dir.
    let lib_path = lib_path.parent().unwrap(); // Where the actual library lives.
    std::fs::copy(lib_path.join(&lib_name), pdx_source_dir.join(&pdex_lib_name))?;

    let pdx_compiler = PathBuf::from(&sdk_path)
        .join("bin")
        .join(format!("pdc{}", EXE_SUFFIX));
    let out = Command::new(&pdx_compiler)
        .current_dir(&pdx_out_dir)
        .args(["-sdkpath", &sdk_path])
        .arg(&pdx_source_dir)
        .arg(&pdx_name)
        .output()?;
    if !out.status.success() {
        Err(PlaydateBuildError::PdxCompilerError(
            String::from_utf8_lossy(&out.stderr).into(),
        ))
    } else {
        Ok(String::from_utf8_lossy(&out.stdout).into())
    }
}

pub fn run_simulator(_pdx_source_dir: &str, pdx_out_dir: &str, pdx_name: &str) -> Result<()> {
    let sdk_path = PathBuf::from(
        std::env::var("PLAYDATE_SDK_PATH")
            .expect("PLAYDATE_SDK_PATH environment variable is not set"),
    );
    let pdx_out_dir = PathBuf::from(pdx_out_dir);
    // This directory, in `pdx_out_dir`, was created by `pdc`, the pdx compiler.
    let pdx = pdx_out_dir.join(format!("{}.pdx", pdx_name));
    let simulator_exe = sdk_path.join("bin").join(crate::consts::SIMULATOR_EXE);
    Command::new(&simulator_exe).arg(pdx).spawn()?;
    Ok(())
}
