fn main() {
  let r = playdate_build::run_simulator(
    env!("PDX_SOURCE_DIR"),
    env!("PDX_OUT_DIR"),
    env!("PDX_NAME"),
  );
  if let Err(e) = r {
    println!("Failed to run simulator\n{}", e);
  }
}
