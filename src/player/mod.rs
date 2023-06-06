pub mod button_states;
pub mod player_manager;
pub mod save_data;
pub mod state;

pub use button_states::ButtonStates;
pub use hex_instance::Instance;
pub use player_manager::PlayerManager;
pub use save_data::SaveData;
pub use state::State;

use crate::{map_manager::Construct, Projectile, HOTBAR_SLOTS, PLAYER_MOVE_SPEED};
use hex::{
    anyhow,
    components::Sprite,
    ecs::{component_manager::Component, Id, Scene},
    id,
    math::Vec2d,
};
use hex_physics::Collider;
use std::time::Instant;

#[derive(Clone)]
pub struct Player<'a> {
    pub health: f32,
    pub fire_time: Instant,
    pub trail_time: Instant,
    pub states: ButtonStates,
    pub trail: (Projectile, Instance),
    pub projectile: (Projectile, Collider, Instance),
    pub hotbar: Vec<Option<(Construct<'a>, Instance, Sprite)>>,
}

impl<'a> Player<'a> {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            health: 25.0,
            fire_time: Instant::now(),
            trail_time: Instant::now(),
            states: Default::default(),
            trail: Projectile::player_trail(scene)?,
            projectile: Projectile::player_bullet(scene)?,
            hotbar: Self::default_hotbar(scene)?,
        })
    }

    pub fn current_item(&self) -> Option<(Construct<'a>, Instance, Sprite)> {
        self.hotbar.get(self.states.mode).cloned()?
    }

    pub fn default_hotbar(
        scene: &Scene,
    ) -> anyhow::Result<Vec<Option<(Construct<'a>, Instance, Sprite)>>> {
        let mut hotbar = vec![None; HOTBAR_SLOTS];

        hotbar[1] = Some(Construct::miner(scene)?);

        Ok(hotbar)
    }

    pub fn force(&self) -> Vec2d {
        let mut force = Vec2d::default();

        if self.states.forward {
            *force.y_mut() += 1.0;
        }

        if self.states.backward {
            *force.y_mut() -= 1.0;
        }

        if self.states.left {
            *force.x_mut() -= 1.0;
        }

        if self.states.right {
            *force.x_mut() += 1.0;
        }

        if force.magnitude() > 0.0 {
            force = force.normal() * PLAYER_MOVE_SPEED;
        }

        force
    }
}

impl Component for Player<'_> {
    fn id() -> Id {
        id!()
    }
}
