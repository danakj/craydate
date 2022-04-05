use std::path::Path;

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

pub fn generate_assets<P: AsRef<Path>>(asset_dir: P, pdx_source_dir: &str) -> Result<(), Error> {
  sync(asset_dir.as_ref(), pdx_source_dir)?;

  // TODO: separate assets into raw assets and cooked assets
  // TODO: build map files here
  Ok(())
}
