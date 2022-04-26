#[cfg(not(feature = "bins"))]
fn main() {
  compile_error!("compile with the feature \"bins\" enabled (`--features=bins`)");
}

#[cfg(feature = "bins")]
fn main() {
  // TODO: Generate assets for your game here, copying them to `env!("PDX_SOURCE_DIR")`.
  // Example:
  //   game_assets::generate_assets(env!("PDX_SOURCE_DIR"))?;

  // Builds the game's pdx image.
  let r = craydate_build::build_pdx(
    env!("PDX_SOURCE_DIR"),
    env!("PDX_OUT_DIR"),
    env!("PDX_NAME"),
  );
  match r {
    Ok(stdout) => println!("{}", stdout),
    Err(e) => println!("Failed\n{}", e),
  }
}
