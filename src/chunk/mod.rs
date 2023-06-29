pub mod chunk_data;
pub mod chunk_manager;
pub mod map;
pub mod ore;
pub mod tile_data;

pub use crate::player::State;
pub use chunk_data::ChunkData;
pub use chunk_manager::ChunkManager;
pub use map::Map;
pub use ore::Ore;
pub use tile_data::TileData;

use crate::CHUNK_SIZE;
use hex::{
    ecs::{component_manager::Component, Id},
    id,
};
use std::rc::Rc;

#[derive(Clone)]
pub struct Chunk {
    pub grid: Vec<Vec<Option<Rc<String>>>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            grid: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        }
    }
}

impl Component for Chunk {
    fn id() -> Id {
        id!()
    }
}
