pub mod asteroid_manager;

pub use asteroid_manager::AsteroidManager;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

pub struct Asteroid {
    pub active: bool,
}

impl Component for Asteroid {
    fn id() -> Id {
        id!()
    }
}
