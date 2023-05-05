use super::Projectile;
use hex::{
    anyhow,
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::Event,
};
use hex_physics::Collider;
use std::time::Instant;

#[derive(Default)]
pub struct ProjectileManager {
    queued_rm: Vec<Id>,
}

impl<'a> System<'a> for ProjectileManager {
    fn update(
        &mut self,
        event: &mut Ev,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = event
        {
            let now = Instant::now();

            while let Some(e) = self.queued_rm.pop() {
                em.rm(e, cm);
            }

            let removed: Vec<_> = em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    let projectile = cm
                        .get::<Projectile>(e, em)
                        .and_then(|p| p.active.then_some(p))?;

                    cm.get::<Collider>(e, em)
                        .map(|collider| {
                            collider
                                .collisions
                                .iter()
                                .cloned()
                                .filter_map(|c| cm.get::<Collider>(c, em))
                                .any(|c| !c.ghost)
                                || now.duration_since(*projectile.spawn_time.get_or_init(|| now))
                                    >= projectile.alive_time
                        })?
                        .then_some(e)
                })
                .collect();

            self.queued_rm.extend(removed);
        }

        Ok(())
    }
}
