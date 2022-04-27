use std::path::PathBuf;
use std::result::Result;

const ASSETS: [&str; 2] = [
    "mojojojo.wav",
    "pirate.mid"
];

pub fn generate_assets<P: Into<PathBuf>>(to: P) -> Result<(), std::io::Error> {
  let to = to.into();

  let from_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
  for a in ASSETS {
    std::fs::copy(from_dir.join(a), to.join(a))?;
  }
  Ok(())
}
