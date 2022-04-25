fn main() {
  if let Err(e) = game_build::generate_assets(env!("PDX_SOURCE_DIR")) {
    println!("Failed to build assets\n{}", e);
    return;
  }

  // Builds the game's pdx image.
  let r = playdate_build::build_pdx(
    env!("PDX_SOURCE_DIR"),
    env!("PDX_OUT_DIR"),
    env!("PDX_NAME"),
  );
  match r {
    Ok(stdout) => println!("{}", stdout),
    Err(e) => println!("Failed\n{}", e),
  }
}
