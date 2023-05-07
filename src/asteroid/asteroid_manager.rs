use super::Asteroid;
use crate::{util, ASTEROID_LAYER, MAP_DIMS_X, MAP_DIMS_Y, PLAYER_LAYER};
use hex::{
    anyhow,
    assets::Texture,
    components::Transform,
    ecs::{system_manager::System, ComponentManager, EntityManager, Ev, Scene},
    math::Vec2d,
};
use hex_instance::Instance;
use hex_physics::{Collider, Physical};
use rand::prelude::*;
use std::rc::Rc;

pub struct AsteroidManager {
    pub asteroid_textures: Vec<Rc<Texture>>,
}

impl AsteroidManager {
    pub fn new(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            asteroid_textures: vec![Rc::new(util::load_texture(
                &scene.display,
                include_bytes!("asteroid.png"),
            )?)],
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
            cm.add(asteroid, Physical::new(Default::default(), true), em);
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
        // Test
        for _ in 0..10 {
            self.spawn_asteroid((em, cm));
        }

        Ok(())
    }

    fn update(
        &mut self,
        _: &mut Ev,
        _: &mut Scene,
        _: (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
