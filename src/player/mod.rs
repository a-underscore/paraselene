pub mod player_manager;
pub mod states;

pub use hex_instance::Instance;
pub use player_manager::PlayerManager;
pub use states::States;

use crate::{util, Projectile, ASTEROID_LAYER, PLAYER_LAYER, PLAYER_MOVE_SPEED, PROJECTILE_LAYER};
use hex::{
    anyhow,
    ecs::{component_manager::Component, Id, Scene},
    id,
    math::Vec2d,
};
use hex_physics::Collider;
use std::time::Instant;

#[derive(Clone)]
pub struct Player {
    pub health: f32,
    pub fire_time: Instant,
    pub states: States,
    pub projectile: (Collider, Projectile, Instance),
}

impl Player {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            health: 25.0,
            fire_time: Instant::now(),
            states: Default::default(),
            projectile: (
                Collider::rect(
                    Vec2d([1.0 / 3.0; 2]),
                    vec![PLAYER_LAYER, ASTEROID_LAYER, PROJECTILE_LAYER],
                    vec![PROJECTILE_LAYER],
                    false,
                    true,
                ),
                Projectile::player_bullet(true),
                Instance::new(
                    util::load_texture(&scene.display, include_bytes!("player_projectile.png"))?,
                    [1.0; 4],
                    0.0,
                    true,
                ),
            ),
        })
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

impl Component for Player {
    fn id() -> Id {
        id!()
    }
}
