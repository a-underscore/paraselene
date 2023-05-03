use super::Projectile;
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
use std::time::Instant;

pub struct ProjectileManager;

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
            for e in em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    let projectile = cm.get::<Projectile>(e, em)?;

                    cm.get::<Collider>(e, em)
                        .map(|collider| {
                            collider
                                .collisions
                                .iter()
                                .cloned()
                                .filter_map(|c| cm.get::<Collider>(c, em))
                                .any(|c| !c.ghost)
                                || Instant::now().duration_since(
                                    *projectile.spawn_time.get_or_init(|| Instant::now()),
                                ) >= projectile.alive_time
                        })?
                        .then_some(e)
                })
                .collect::<Vec<_>>()
            {
                em.rm(e, cm);
            }
        }

        Ok(())
    }
}
