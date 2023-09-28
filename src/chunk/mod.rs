pub mod chunk_data;
pub mod chunk_manager;
pub mod map;
pub mod tile;
pub mod tile_data;

pub use crate::player::State;
pub use chunk_data::ChunkData;
pub use chunk_manager::ChunkManager;
pub use map::Map;
pub use tile::Tile;
pub use tile_data::TileData;

use hex::ecs::component_manager::Component;

pub const CHUNK_SIZE: u32 = 8;

#[derive(Clone)]
pub struct Chunk {
    pub grid: Vec<Vec<Option<String>>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            grid: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        }
    }
}

impl Component for Chunk {}
