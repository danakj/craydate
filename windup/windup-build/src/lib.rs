use std::path::{Path, PathBuf};

use anyhow::Error;

extern crate rusync;

fn sync<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<(), Error> {
  let options = rusync::SyncOptions::default();
  let progress_info = Box::new(rusync::ConsoleProgressInfo::new());
  let syncer = rusync::Syncer::new(
    source.as_ref(),
    destination.as_ref(),
    options,
    progress_info,
  );
  Ok(syncer.sync()?).map(|_| ())
}

pub fn generate_assets(pdx_source_dir: &str) -> Result<(), Error> {
  const RAW_ASSET_DIR: &str = "assets/raw/";

  let windup_build_dir = env!("CARGO_MANIFEST_DIR");
  let raw_asset_dir = PathBuf::from(windup_build_dir).join(RAW_ASSET_DIR);

  sync(raw_asset_dir, pdx_source_dir)?;

  // TODO: build map files here
  Ok(())
}
