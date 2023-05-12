use super::Asteroid;
use crate::{util, ASTEROID_LAYER, MAP_DIMS_X, MAP_DIMS_Y, NUM_ASTEROIDS, PLAYER_LAYER};
use hex::{
    anyhow,
    assets::Texture,
    components::Transform,
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Id, Scene},
    glium::glutin::event::Event,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::Collider;
use rand::prelude::*;
use std::rc::Rc;

#[derive(Default)]
pub struct AsteroidManager {
    pub asteroid_textures: Vec<Rc<Texture>>,
    pub player: OnceCell<Option<Id>>,
}

impl AsteroidManager {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            asteroid_textures: vec![
                Rc::new(util::load_texture(
                    &scene.display,
                    include_bytes!("asteroid.png"),
                )?),
                Rc::new(util::load_texture(
                    &scene.display,
                    include_bytes!("asteroid2.png"),
                )?),
            ],
            ..Default::default()
        })
    }

    pub fn spawn_asteroid(&mut self, (em, cm): (&mut EntityManager, &mut ComponentManager)) {
        if let Some(texture) = self
            .asteroid_textures
            .choose(&mut rand::thread_rng())
            .cloned()
        {
            let mut rng = rand::thread_rng();
            let asteroid = em.add();

            cm.add(
                asteroid,
                Instance {
                    texture,
                    color: [1.0; 4],
                    z: 1.0,
                    active: true,
                },
                em,
            );
            cm.add(
                asteroid,
                Collider::oct(
                    Vec2d([1.0 / 3.0; 2]),
                    vec![PLAYER_LAYER, ASTEROID_LAYER],
                    Vec::new(),
                    false,
                    true,
                ),
                em,
            );
            cm.add(
                asteroid,
                Transform::new(
                    Vec2d::new(
                        rng.gen_range((-MAP_DIMS_X / 2.0)..(MAP_DIMS_Y / 2.0)),
                        rng.gen_range((-MAP_DIMS_X / 2.0)..(MAP_DIMS_Y / 2.0)),
                    ),
                    0.0,
                    Vec2d([1.0; 2]),
                    true,
                ),
                em,
            );
            cm.add(asteroid, Asteroid::new(true), em);
        }
    }
}

impl<'a> System<'a> for AsteroidManager {
    fn init(
        &mut self,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        for _ in 0..NUM_ASTEROIDS {
            self.spawn_asteroid((em, cm));
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
            let asteroids: Vec<_> = em
                .entities
                .keys()
                .cloned()
                .filter(|e| {
                    cm.get::<Asteroid>(*e, em)
                        .and_then(|a| a.active.then_some(a))
                        .is_some()
                })
                .collect();
            let colliders: Vec<_> = em
                .entities
                .keys()
                .cloned()
                .filter(|e| !asteroids.contains(e))
                .filter_map(|e| {
                    cm.get::<Collider>(e, em).and_then(|c| {
                        Some((
                            c.active.then_some(c.boundary)?,
                            cm.get::<Transform>(e, em)
                                .and_then(|t| t.active.then_some(t.position()))?,
                        ))
                    })
                })
                .collect();

            for e in asteroids {
                if let Some((position, collider)) =
                    cm.get::<Transform>(e, em).cloned().and_then(|t| {
                        Some((
                            t.active.then_some(t.position())?,
                            cm.get_mut::<Collider>(e, em)?,
                        ))
                    })
                {
                    collider.active = colliders
                        .iter()
                        .cloned()
                        .any(|(b, p)| (position - p).magnitude() <= collider.boundary + b);
                }
            }
        }

        Ok(())
    }
}
