pub mod player_manager;

pub use player_manager::PlayerManager;

pub use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

pub struct Player {
    pub health: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { health: 25.0 }
    }
}

impl Component for Player {
    fn id() -> Id {
        id!()
    }
}
