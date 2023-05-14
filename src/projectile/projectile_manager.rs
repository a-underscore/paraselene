use super::Projectile;
use crate::Tag;
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::Event,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::Collider;
use std::time::Instant;

#[derive(Default)]
pub struct ProjectileManager {
    pub player: OnceCell<Option<Id>>,
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

            let projectiles: Vec<_> = em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    let projectile = cm
                        .get::<Projectile>(e, em)
                        .and_then(|p| p.active.then_some(p))?;
                    let spawn_time = *projectile.spawn_time.get_or_init(|| now);

                    Some((e, spawn_time, projectile.clone()))
                })
                .collect();
            let (_, mut trail) = projectiles.into_iter().fold(
                (&mut self.queued_rm, Vec::new()),
                |(rm, mut trail), (e, spawn_time, projectile)| {
                    let delta = now.duration_since(spawn_time);
                    let t = projectile.trail_data.and_then(|t| {
                        let scale = 1.0 - now.duration_since(spawn_time).as_secs_f32() * t;

                        if let Some(instance) = cm.get_mut::<Instance>(e, em) {
                            instance.color[3] = scale;
                        }

                        if let Some(transform) = cm.get_mut::<Transform>(e, em) {
                            transform.set_scale(Vec2d([scale; 2]));
                        }

                        Some(t)
                    });
                    let r = (cm
                        .get::<Collider>(e, em)
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
                        .then_some(e);

                    if let Some(r) = r {
                        rm.push(r);
                    } else if let Some(t) = t
                        .is_some()
                        .then_some(e)
                        .and_then(|e| cm.get::<Transform>(e, em))
                    {
                        trail.push((e, t.position()));
                    }

                    (rm, trail)
                },
            );

            if let Some(player_pos) = self
                .player
                .get_or_init(|| Tag::new("player").find((em, cm)))
                .and_then(|p| {
                    cm.get::<Transform>(p, em)
                        .and_then(|t| t.active.then_some(t.position()))
                })
            {
                trail.sort_by(|(_, p1), (_, p2)| {
                    (player_pos - *p1)
                        .magnitude()
                        .total_cmp(&(player_pos - *p2).magnitude())
                });

                for t in trail.windows(2) {
                    if let [(e1, _), (_, p2)] = t {
                        if let Some(t) = cm
                            .get_mut::<Transform>(*e1, em)
                            .and_then(|t| t.active.then_some(t))
                        {
                            t.set_rotation(Vec2d::new(0.0, 1.0).angle(t.position() - *p2));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
