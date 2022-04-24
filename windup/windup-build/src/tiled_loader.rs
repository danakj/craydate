use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};

#[derive(Clone, Copy)]
pub struct Extents {
  pub min_x: i32,
  pub max_x: i32,
  pub min_y: i32,
  pub max_y: i32,
}

// TODO: tiled should provide this and not just make Chunk private, yikes /o\
struct LayerIter<'a> {
  extents: Extents,
  layer: &'a tiled::InfiniteTileLayer<'a>,
  x: i32,
  y: i32,
}
impl<'a> LayerIter<'a> {
  pub fn new(extents: Extents, layer: &'a tiled::InfiniteTileLayer<'a>) -> Self {
    Self {
      extents,
      layer,
      x: extents.min_x,
      y: extents.min_y,
    }
  }
}
impl<'a> Iterator for LayerIter<'a> {
  type Item = (tiled::LayerTile<'a>, i32, i32);

  fn next(&mut self) -> Option<Self::Item> {
    if self.y > self.extents.max_y {
      return None;
    }

    loop {
      let orig_x = self.x;
      let orig_y = self.y;
      let tile = self.layer.get_tile(self.x, self.y);

      self.x += 1;
      if self.x > self.extents.max_x {
        self.y += 1;
        self.x = self.extents.min_x;
      }
      if self.y > self.extents.max_y {
        return None;
      }

      if let Some(tile) = tile {
        return Some((tile, orig_x, orig_y));
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (
      0,
      Some(
        (self.extents.max_x + 1 - self.extents.min_x) as usize
          * (self.extents.max_y + 1 - self.extents.min_y) as usize,
      ),
    )
  }
}
impl core::iter::FusedIterator for LayerIter<'_> {}

pub fn relative_image_path(path: &PathBuf) -> Option<String> {
  let source = path.canonicalize().unwrap();
  let source = source.into_os_string().into_string().unwrap();
  let mut source = source.replace("\\", "/");

  // Delete everything before (and including) PREFIX.
  const PREFIX: &str = "windup-build/assets/raw/";
  match source.find(PREFIX) {
    Some(prefix_idx) => {
      source.replace_range(0..prefix_idx + PREFIX.len(), "");
      Some(source)
    }
    None => None,
  }
}

fn load(tmx_map_file: &Path, extents: Extents) -> Result<windup_map::Map, Error> {
  let mut output = windup_map::Map {
    tiles: Vec::new(),
    layers: Vec::new(),
  };
  output.tiles.push(windup_map::TileData { path: None });
  let invalid_tile_id = windup_map::TileId(0);

  let mut tile_map: HashMap<(usize, u32), windup_map::TileId> = HashMap::new();

  let mut loader = tiled::Loader::new();
  let src_map = loader.load_tmx_map(tmx_map_file)?;

  for (set_idx, tileset) in src_map.tilesets().iter().enumerate() {
    for (id, tile) in tileset.tiles() {
      let mapped_id = windup_map::TileId(output.tiles.len().try_into().unwrap());

      let path = match &tile.image {
        Some(image) => relative_image_path(&image.source),
        None => None,
      };

      output.tiles.push(windup_map::TileData { path });
      tile_map.insert((set_idx, id), mapped_id);
    }
  }

  for layer in src_map.layers() {
    let layer = match layer.layer_type() {
      tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(x)) => x,
      _ => continue,
    };
    let mut output_layer = windup_map::Layer { blocks: Vec::new() };

    for (tile, x, y) in LayerIter::new(extents, &layer) {
      let tile_id = match tile_map.get(&(tile.tileset_index(), tile.id())) {
        Some(tile_id) => tile_id,
        None => &invalid_tile_id,
      };
      output_layer.blocks.push(windup_map::LayerTile { x, y, id: *tile_id });
    }
    output.layers.push(output_layer);
  }

  Ok(output)
}

fn write(map: windup_map::Map, filename: &Path) -> Result<(), Error> {
  let bytes = map.to_vec().map_err(|e| anyhow!(e))?;
  fs::write(filename, bytes).map_err(|e| anyhow!(e))
}

pub fn write_map<P: AsRef<Path>, Q: AsRef<Path>>(
  tmx_map_file: P,
  extents: Extents,
  output_file: Q,
) -> Result<(), Error> {
  let map = load(tmx_map_file.as_ref(), extents)?;
  write(map, output_file.as_ref())
}
