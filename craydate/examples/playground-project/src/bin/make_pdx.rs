#[cfg(not(feature = "bins"))]
fn main() {
  compile_error!("compile with the feature \"bins\" enabled (`--features=bins`)");
}

#[cfg(feature = "bins")]
fn main() {
  let srcdir = env!("PDX_SOURCE_DIR");

  if !std::path::PathBuf::from(srcdir).exists() {
    if let Err(e) = std::fs::create_dir(srcdir) {
      println!("Failed making PDX_SOURCE_DIR dir ({}): {}", srcdir, e);
    }
  }

  if let Err(e) = game_assets::generate_assets(srcdir) {
    println!("Failed generating assets\n{}", e);
  }

  // Builds the game's pdx image.
  let r = craydate_build::build_pdx(srcdir, env!("PDX_OUT_DIR"), env!("PDX_NAME"));
  match r {
    Ok(stdout) => println!("{}", stdout),
    Err(e) => println!("Failed\n{}", e),
  }
}
