pub mod projectile_manager;

pub use projectile_manager::ProjectileManager;

use crate::{util, PLAYER_LAYER, PROJECTILE_LAYER};
use hex::{
    anyhow,
    ecs::{component_manager::Component, Id, Scene},
    id,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::Collider;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Projectile {
    pub spawn_time: OnceCell<Instant>,
    pub alive_time: Duration,
    pub velocity: Vec2d,
    pub cooldown: Duration,
    pub trail_data: Option<f32>,
    pub dmg: f32,
}

impl Projectile {
    pub fn player_bullet(scene: &Scene) -> anyhow::Result<(Self, Collider, Instance)> {
        Ok((
            Self {
                spawn_time: OnceCell::new(),
                alive_time: Duration::from_secs_f32(1.0),
                velocity: Vec2d::new(0.0, 30.0),
                cooldown: Duration::from_millis(50),
                trail_data: None,
                dmg: 2.0,
            },
            Collider::rect(
                Vec2d([1.0 / 3.0; 2]),
                vec![PLAYER_LAYER, PROJECTILE_LAYER],
                vec![PROJECTILE_LAYER],
                false,
                true,
            ),
            Instance::new(
                util::load_texture(&scene.display, include_bytes!("player_projectile.png"))?,
                [1.0; 4],
                -1.0,
                true,
            ),
        ))
    }
}

impl Component for Projectile {
    fn id() -> Id {
        id!()
    }
}
