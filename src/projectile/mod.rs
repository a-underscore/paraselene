pub mod projectile_manager;

pub use projectile_manager::ProjectileManager;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Projectile {
    pub spawn_time: OnceCell<Instant>,
    pub alive_time: Duration,
    pub velocity: Vec2d,
    pub cooldown: Duration,
    pub vis_mul: Option<f32>,
    pub dmg: f32,
    pub active: bool,
}

impl Projectile {
    pub fn player_bullet(active: bool) -> Self {
        Self {
            spawn_time: OnceCell::new(),
            alive_time: Duration::from_secs_f32(2.0),
            velocity: Vec2d::new(0.0, 30.0),
            cooldown: Duration::from_millis(36),
            vis_mul: None,
            dmg: 2.0,
            active,
        }
    }

    pub fn player_trail(active: bool) -> Self {
        Self {
            spawn_time: OnceCell::new(),
            alive_time: Duration::from_secs_f32(0.5),
            velocity: Vec2d::new(0.0, 0.0),
            cooldown: Duration::from_millis(1),
            vis_mul: Some(2.0),
            dmg: 0.0,
            active,
        }
    }
}

impl Component for Projectile {
    fn id() -> Id {
        id!()
    }
}
