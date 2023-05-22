pub mod asteroid_manager;
pub mod ore;

pub use asteroid_manager::AsteroidManager;
pub use ore::Ore;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

pub struct Asteroid {
    pub active: bool,
}

impl Asteroid {
    pub fn new(active: bool) -> Self {
        Self { active }
    }
}

impl Component for Asteroid {
    fn id() -> Id {
        id!()
    }
}
