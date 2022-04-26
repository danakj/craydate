#[cfg(target_os = "windows")]
pub const SIMULATOR_EXE: &str = "PlaydateSimulator.exe";
#[cfg(target_os = "mac")]
pub const SIMULATOR_EXE: &str = compiler_error!("What is the simulator execuable name?");
#[cfg(target_os = "linux")]
pub const SIMULATOR_EXE: &str = compiler_error!("What is the simulator execuable name?");
