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
use hex_instance::Instance;
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
                    let (spawn_time, projectile) = {
                        let projectile = cm
                            .get::<Projectile>(e, em)
                            .and_then(|p| p.active.then_some(p))?;

                        let spawn_time = *projectile.spawn_time.get_or_init(|| now);
                        (spawn_time, projectile.clone())
                    };
                    let delta = now.duration_since(spawn_time);

                    if let Some(instance) = cm
                        .get_mut::<Instance>(e, em)
                        .and_then(|i| i.active.then_some(i))
                    {
                        if let Some(vis_mul) = projectile
                            .vis_mul
                            .map(|v| (1.0 - now.duration_since(spawn_time).as_secs_f32() * v))
                        {
                            instance.color[3] = vis_mul;
                        }
                    }

                    (cm.get::<Collider>(e, em)
                        .map(|collider| {
                            collider
                                .collisions
                                .iter()
                                .cloned()
                                .filter_map(|c| cm.get::<Collider>(c, em))
                                .any(|c| !c.ghost)
                        })
                        .unwrap_or(false)
                        || delta >= projectile.alive_time)
                        .then_some(e)
                })
                .collect();

            self.queued_rm.extend(removed);
        }

        Ok(())
    }
}
