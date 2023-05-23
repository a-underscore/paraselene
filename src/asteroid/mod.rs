pub mod asteroid_manager;
pub mod chunk;
pub mod ore;

pub use asteroid_manager::AsteroidManager;
pub use chunk::Chunk;
pub use ore::Ore;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
};
use std::rc::Rc;

pub struct Asteroid {
    pub active: bool,
    pub id: Rc<String>,
}

impl Asteroid {
    pub fn new(id: Rc<String>, active: bool) -> Self {
        Self { id, active }
    }
}

impl Component for Asteroid {
    fn id() -> Id {
        id!()
    }
}
