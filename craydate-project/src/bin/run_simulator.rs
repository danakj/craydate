#[cfg(not(feature = "bins"))]
fn main() {
  compile_error!("compile with the feature \"bins\" enabled (`--features=bins`)");
}

#[cfg(feature = "bins")]
fn main() {
  let r = craydate_build::run_simulator(
    env!("PDX_SOURCE_DIR"),
    env!("PDX_OUT_DIR"),
    env!("PDX_NAME"),
  );
  if let Err(e) = r {
    println!("Failed to run simulator\n{}", e);
  }
}
