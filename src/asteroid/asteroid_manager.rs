use super::Asteroid;
use crate::Projectile;
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
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            for e in em.entities.keys().cloned() {
                if let Some((c, a_id)) = cm
                    .get::<Collider>(e, em)
                    .cloned()
                    .and_then(|c| Some((c.active.then_some(c)?, cm.get_id::<Asteroid>(e, em)?)))
                {
                    if c.collisions.iter().cloned().any(|c| {
                        cm.get::<Projectile>(e, em)
                            .and_then(|p| p.active.then_some(c))
                            .is_some()
                    }) {
                        if let Some(a) = cm.get_cache_mut::<Asteroid>(a_id) {
                            a.order -= 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
