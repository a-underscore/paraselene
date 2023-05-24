use super::TileData;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub position: [f32; 2],
    pub tiles: Vec<TileData>,
}
