use alloc::string::String;
use alloc::vec::Vec;

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TileId(pub i32);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct TileData {
  pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerTile {
  pub x: i32,
  pub y: i32,
  pub id: TileId,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Layer {
  pub blocks: Vec<LayerTile>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Map {
  pub tiles: Vec<TileData>,
  pub layers: Vec<Layer>,
}

impl Map {
  pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
    postcard::to_allocvec(&self).map_err(|e| anyhow!(e))
  }
}
