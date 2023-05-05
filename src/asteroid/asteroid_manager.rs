use super::Asteroid;
use crate::Projectile;

use hex::{
    anyhow,
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Scene,
    },
    glium::glutin::event::Event,
};

use hex_physics::Collider;

pub struct AsteroidManager;

impl System<'_> for AsteroidManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _scene: &mut Scene,
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
