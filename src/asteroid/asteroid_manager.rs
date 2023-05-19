use super::Asteroid;
use crate::{util, MAP_DIMS_X, MAP_DIMS_Y};
use hex::{
    anyhow,
    assets::Texture,
    components::Transform,
    ecs::{system_manager::System, ComponentManager, EntityManager, Id, Scene},
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use noise::{NoiseFn, Perlin};
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

    pub fn spawn_asteroid(
        &mut self,
        pos: Vec2d,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        if let Some(texture) = self
            .asteroid_textures
            .choose(&mut rand::thread_rng())
            .cloned()
        {
            let asteroid = em.add();

            cm.add(
                asteroid,
                Instance {
                    texture,
                    color: [1.0; 4],
                    z: -2.0,
                    active: true,
                },
                em,
            );
            cm.add(
                asteroid,
                Transform::new(pos, 0.0, Vec2d([1.0; 2]), true),
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
        let perlin = Perlin::new(thread_rng().gen_range(0..u32::MAX));

        for i in 0..(MAP_DIMS_X as u32) {
            for j in 0..(MAP_DIMS_Y as u32) {
                let val = perlin.get([i as f64 * 1.0 / 25.0, j as f64 * 1.0 / 25.0, 0.0]);

                if val > 0.5 {
                    self.spawn_asteroid(Vec2d::new(i as f32, j as f32), (em, cm));
                }
            }
        }

        Ok(())
    }
}
