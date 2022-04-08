use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug)]
pub struct TileId(pub i32);

#[derive(Debug)]
pub struct TileData {
  pub path: Option<String>,
}

#[derive(Debug)]
pub struct LayerTile {
  pub x: i32,
  pub y: i32,
  pub id: TileId,
}

#[derive(Debug)]
pub struct Layer {
  pub blocks: Vec<LayerTile>,
}

#[derive(Debug)]
pub struct Map {
  pub tiles: Vec<TileData>,
  pub layers: Vec<Layer>,
}
