pub mod asteroid_manager;

pub use asteroid_manager::AsteroidManager;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

#[derive(Clone)]
pub struct Asteroid {
    pub order: u32,
    pub active: bool,
}

impl Asteroid {
    pub fn large_asteroid(active: bool) -> Self {
        Self { order: 1, active }
    }

    pub fn small_asteroid(active: bool) -> Self {
        Self { order: 0, active }
    }
}

impl Component for Asteroid {
    fn id() -> Id {
        id!()
    }
}
