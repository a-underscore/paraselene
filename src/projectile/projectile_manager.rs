use super::Projectile;
use hex::{
    anyhow,
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, Context, EntityManager, Id,
    },
    glium::glutin::event::Event,
};
use hex_physics::Collider;
use std::time::Instant;

#[derive(Default)]
pub struct ProjectileManager {
    queued_rm: Vec<Id>,
}

impl System for ProjectileManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();

            while let Some(e) = self.queued_rm.pop() {
                em.rm(e, cm);
            }

            let projectiles: Vec<_> = em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    let projectile = cm.get::<Projectile>(e, em)?;
                    let spawn_time = *projectile.spawn_time.get_or_init(|| now);

                    Some((e, spawn_time, projectile.clone()))
                })
                .collect();
            let rm: Vec<_> = projectiles
                .into_iter()
                .filter_map(|(e, spawn_time, projectile)| {
                    let delta = now.duration_since(spawn_time);

                    (delta >= projectile.alive_time
                        || cm
                            .get::<Collider>(e, em)
                            .map(|collider| {
                                collider
                                    .collisions
                                    .iter()
                                    .cloned()
                                    .filter_map(|c| cm.get::<Collider>(c, em))
                                    .any(|c| !c.ghost)
                            })
                            .unwrap_or(false))
                    .then_some(e)
                })
                .collect();

            self.queued_rm.extend(rm);
        }

        Ok(())
    }
}
