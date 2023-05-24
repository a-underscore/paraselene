use crate::CHUNK_SIZE;
use hex::math::Vec2d;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub position: [f32; 2],
    pub grid: Vec<Vec<Option<String>>>,
}

impl ChunkData {
    pub fn new(position: Vec2d) -> Self {
        Self {
            position: position.0,
            grid: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        }
    }
}
