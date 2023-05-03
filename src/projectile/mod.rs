pub mod projectile_manager;

use hex::{
    ecs::{component_manager::Component, Id},
    id,
    math::Vec2d,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Projectile {
    pub spawn_time: Instant,
    pub alive_time: Duration,
    pub velocity: Vec2d,
    pub cooldown: Duration,
    pub dmg: f32,
}

impl Projectile {
    pub fn player_bullet() -> Self {
        Self {
            spawn_time: Instant::now(),
            alive_time: Duration::from_secs_f32(2.0),
            velocity: Vec2d::new(0.0, 30.0),
            cooldown: Duration::from_millis(36),
            dmg: 2.0,
        }
    }
}

impl Component for Projectile {
    fn id() -> Id {
        id!()
    }
}
