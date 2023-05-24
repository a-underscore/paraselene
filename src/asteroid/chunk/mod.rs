pub mod chunk_data;
pub mod tile_data;

pub use chunk_data::ChunkData;
pub use tile_data::TileData;

use crate::CHUNK_SIZE;
use hex::{
    ecs::{component_manager::Component, Id},
    id,
};
use std::rc::Rc;

pub struct Chunk {
    pub grid: Vec<Vec<Option<Rc<String>>>>,
    pub active: bool,
}

impl Chunk {
    pub fn new(active: bool) -> Self {
        Self {
            grid: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            active,
        }
    }
}

impl Component for Chunk {
    fn id() -> Id {
        id!()
    }
}
