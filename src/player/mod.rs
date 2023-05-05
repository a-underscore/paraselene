pub mod player_manager;

pub use hex_instance::Instance;
pub use player_manager::PlayerManager;

use crate::{util, Projectile, PLAYER_LAYER, PROJECTILE_LAYER};
use hex::{
    anyhow,
    ecs::{component_manager::Component, Id, Scene},
    id,
    math::Vec2d,
};
use hex_physics::Collider;
use std::time::Instant;

pub struct Player {
    pub health: f32,
    pub firing: bool,
    pub fire_time: Instant,
    pub projectile: (Collider, Projectile, Instance),
}

impl Player {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            health: 25.0,
            firing: false,
            fire_time: Instant::now(),
            projectile: (
                Collider::oct(
                    Vec2d::new(1.0 / 3.0, 1.0 / 3.0),
                    Vec2d::new(1.0 / 3.0, 1.0 / 3.0).magnitude(),
                    vec![PLAYER_LAYER, PROJECTILE_LAYER],
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
}

impl Component for Player {
    fn id() -> Id {
        id!()
    }
}
