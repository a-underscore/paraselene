use super::Asteroid;
use crate::{util, Tag, ASTEROID_UPDATE_TIME, CAM_DIMS, MAP_DIMS_X, MAP_DIMS_Y};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Id, Scene},
    glium::glutin::event::Event,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use std::time::Instant;

pub struct AsteroidManager {
    pub instance: Instance,
    pub player: OnceCell<Option<Id>>,
    pub check: Instant,
}

impl AsteroidManager {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            instance: Instance::new(
                util::load_texture(&scene.display, include_bytes!("asteroid.png"))?,
                [1.0; 4],
                -2.0,
                true,
            ),
            player: OnceCell::new(),
            check: Instant::now(),
        })
    }

    pub fn spawn_asteroid(
        &mut self,
        pos: Vec2d,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        let asteroid = em.add();

        cm.add(asteroid, self.instance.clone(), em);
        cm.add(
            asteroid,
            Transform::new(pos, 0.0, Vec2d([1.0; 2]), true),
            em,
        );
        cm.add(asteroid, Asteroid::new(true), em);
    }
}

impl<'a> System<'a> for AsteroidManager {
    fn init(
        &mut self,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        let perlin = Perlin::new(thread_rng().gen_range(0..u32::MAX));

        for i in 0..(MAP_DIMS_X as u32) {
            for j in 0..(MAP_DIMS_Y as u32) {
                let val = perlin.get([i as f64 * 1.0 / 25.0, j as f64 * 1.0 / 25.0, 0.0]);

                if val > 0.25 {
                    self.spawn_asteroid(Vec2d::new(i as f32, j as f32), (em, cm));
                }
            }
        }

        Ok(())
    }

    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();

            if now.duration_since(self.check) >= ASTEROID_UPDATE_TIME {
                self.check = now;

                if let Some(player_pos) = self
                    .player
                    .get_or_init(|| Tag::new("player").find((em, cm)))
                    .and_then(|p| {
                        cm.get::<Transform>(p, em)
                            .and_then(|t| t.active.then_some(t.position()))
                    })
                {
                    for e in em.entities.keys().cloned() {
                        if cm
                            .get::<Asteroid>(e, em)
                            .and_then(|a| a.active.then_some(a))
                            .is_some()
                        {
                            if let Some((position, instance)) =
                                cm.get::<Transform>(e, em).cloned().and_then(|t| {
                                    Some((
                                        t.active.then_some(t.position())?,
                                        cm.get_mut::<Instance>(e, em)?,
                                    ))
                                })
                            {
                                instance.active = (position - player_pos).magnitude()
                                    < (CAM_DIMS.powi(2) * 2.0).sqrt();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
