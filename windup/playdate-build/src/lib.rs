#![deny(clippy::all)]

/// Consts used to configure behaviour that may be controlled by cfgs.
mod consts;
/// Errors that can be returned from the crate.
mod error;

use std::env::consts::EXE_SUFFIX;
use std::path::PathBuf;
use std::process::Command;

pub use error::{PlaydateBuildError, Result};

pub const WINDOWS: (&str, &str) = ("", ".dll");
pub const LINUX: (&str, &str) = ("lib", ".so");
pub const MAC: (&str, &str) = ("lib", ".dylib");
pub const DEVICE: (&str, &str) = ("lib", ".a");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetPlatform {
  Windows,
  Linux,
  Mac,
  Device,
}
impl TargetPlatform {
  fn parse(s: &str) -> std::result::Result<Self, String> {
    Ok(match s {
      "windows" => Self::Windows,
      "linux" => Self::Linux,
      "mac" => Self::Mac,
      "device" => Self::Device,
      _ => Err("unknown platform, should be one of: windows, linux, mac, device".to_string())?,
    })
  }
  const fn lib_prefix(self) -> &'static str {
    match self {
      Self::Windows => WINDOWS.0,
      Self::Linux => LINUX.0,
      Self::Mac => MAC.0,
      Self::Device => DEVICE.0,
    }
  }
  const fn lib_suffix(self) -> &'static str {
    match self {
      Self::Windows => WINDOWS.1,
      Self::Linux => LINUX.1,
      Self::Mac => MAC.1,
      Self::Device => DEVICE.1,
    }
  }
}

fn pdx_name() -> String {
  std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME envionment variable is not set")
}

/// Export variables that will be consumed by Cargo to build a game.
///
/// `path_to_assets` is the relative path from the executable crate's root to the game's assets.
pub fn export_vars() {
  println!("cargo:rustc-env={}={}", "PDX_SOURCE_DIR", "pdx_source");
  println!("cargo:rustc-env={}={}", "PDX_OUT_DIR", "pdx_out",);
  println!("cargo:rustc-env={}={}", "PDX_NAME", pdx_name());
}

pub fn build_pdx(pdx_source_dir: &str, pdx_out_dir: &str, pdx_name: &str) -> Result<String> {
  let sdk_path =
    std::env::var("PLAYDATE_SDK_PATH").expect("PLAYDATE_SDK_PATH environment variable is not set");

  let platform_str = std::env::var("PLAYDATE_TARGET_PLATFORM")
    .expect("PLAYDATE_TARGET_PLATFORM environment variable is not set");
  let platform = TargetPlatform::parse(&platform_str)?;

  let pdx_source_dir = PathBuf::from(pdx_source_dir);
  let pdx_out_dir = PathBuf::from(pdx_out_dir);

  std::fs::create_dir_all(&pdx_source_dir)?;
  std::fs::create_dir_all(&pdx_out_dir)?;

  // Touch the source pdx.bin file, which is empty for the simulator target.
  std::fs::write(pdx_source_dir.join("pdex.bin"), "")?;

  // Copy the library into the source dir for the compiler.
  let lib_name = format!(
    "{}{}{}",
    platform.lib_prefix(),
    pdx_name.replace('-', "_"),
    platform.lib_suffix()
  );
  let pdex_lib_name = format!("{}{}", "pdex", platform.lib_suffix());
  let lib_path = &pdx_out_dir;
  let lib_path = lib_path.parent().unwrap(); // Where the actual library lives.

  std::fs::copy(
    lib_path.join(&lib_name),
    pdx_source_dir.join(&pdex_lib_name),
  )?;

  // The source and out dirs are both relative from the current directory. So find the path back
  // from the out dir to the source dir.
  let mut relpath_to_pdx_source_dir = pdx_source_dir;
  for _ in 0..pdx_out_dir.components().count() {
    relpath_to_pdx_source_dir = PathBuf::from("..").join(relpath_to_pdx_source_dir);
  }

  let pdx_compiler = PathBuf::from(&sdk_path).join("bin").join(format!("pdc{}", EXE_SUFFIX));
  let out = Command::new(&pdx_compiler)
    .current_dir(&pdx_out_dir)
    .args(["-sdkpath", &sdk_path])
    .arg(&relpath_to_pdx_source_dir)
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
    std::env::var("PLAYDATE_SDK_PATH").expect("PLAYDATE_SDK_PATH environment variable is not set"),
  );
  let pdx_out_dir = PathBuf::from(pdx_out_dir);
  // This directory, in `pdx_out_dir`, was created by `pdc`, the pdx compiler.
  let pdx = pdx_out_dir.join(format!("{}.pdx", pdx_name));
  let abs_pdx = std::env::current_dir()?.join(pdx);
  let simulator_exe = sdk_path.join("bin").join(crate::consts::SIMULATOR_EXE);
  Command::new(&simulator_exe).arg(abs_pdx).current_dir(sdk_path).spawn()?;
  Ok(())
}
