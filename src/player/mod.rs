pub mod button_states;
pub mod player_manager;
pub mod save_data;
pub mod state;

pub use button_states::ButtonStates;
pub use hex_instance::Instance;
pub use player_manager::PlayerManager;
pub use save_data::SaveData;
pub use state::State;

use crate::{construct::Construct, projectile::Projectile};
use hex::{
    anyhow,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Id, Scene,
    },
    id,
    math::Vec2d,
};
use hex_physics::Collider;
use std::time::Instant;

pub const HOTBAR_SLOTS: usize = 10;
pub const PLAYER_MOVE_SPEED: f32 = 10.0;

#[derive(Clone)]
pub struct Player<'a> {
    pub health: f32,
    pub fire_time: Instant,
    pub trail_time: Instant,
    pub states: ButtonStates,
    pub projectile: (Projectile, Collider, Instance),
    pub hotbar: Vec<Option<(Construct<'a>, Instance)>>,
}

impl<'a> Player<'a> {
    pub fn new(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        Ok(Self {
            health: 25.0,
            fire_time: Instant::now(),
            trail_time: Instant::now(),
            states: Default::default(),
            projectile: Projectile::player_bullet(scene)?,
            hotbar: Self::default_hotbar(scene, (em, cm))?,
        })
    }

    pub fn current_item(&self) -> Option<(Construct<'a>, Instance)> {
        self.hotbar.get(self.states.mode).cloned().flatten()
    }

    pub fn default_hotbar(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Vec<Option<(Construct<'a>, Instance)>>> {
        let mut hotbar = vec![None; HOTBAR_SLOTS];

        hotbar[1] = Construct::miner(scene, (em, cm))?;

        Ok(hotbar)
    }

    pub fn force(&self) -> Vec2d {
        let mut force = Vec2d::default();

        if self.states.forward {
            force.set_y(force.y() + 1.0);
        }

        if self.states.backward {
            force.set_y(force.y() - 1.0);
        }

        if self.states.left {
            force.set_x(force.x() - 1.0);
        }

        if self.states.right {
            force.set_x(force.x() + 1.0);
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
