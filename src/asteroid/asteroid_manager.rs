use super::Asteroid;
use crate::{util, Tag, PLAYER_LAYER, PROJECTILE_LAYER};
use hex::{
    anyhow,
    components::{Camera, Sprite, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::Event,
    math::{Mat3d, Vec2d},
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::{Collider, Physical};
use hex_ui::ScreenPos;
use std::time::Instant;

pub struct AsteroidManager;

impl System<'_> for AsteroidManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        for e in em.entities.keys().cloned() {
            if let Some((a, s, t, c)) = cm.get::<Asteroid>(e, em).and_then(|a| {
                Some((
                    a.active.then_some(a)?,
                    cm.get::<Sprite>(e, em)?,
                    cm.get::<Transform>(e, em)?,
                    cm.get::<Collider>(e, em)?,
                ))
            }) {

            }
        }

        Ok(())
    }
}
