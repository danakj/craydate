use std::path::PathBuf;

const MANIFEST_TO_ASSET_DIR: &str = "../windup/assets/";

fn main() {
  let sim_manifest_dir = env!("SIM_MANIFEST_DIR");
  let asset_dir = PathBuf::from(sim_manifest_dir).join(MANIFEST_TO_ASSET_DIR);
  if let Err(e) = asset_build::generate_assets(asset_dir, env!("PDX_SOURCE_DIR")) {
    println!("Failed to build assets\n{}", e);
    return;
  }

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
