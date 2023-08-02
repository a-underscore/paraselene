use crate::util;
use hex::{anyhow, assets::Texture, ecs::Scene};
use rand::prelude::*;

pub const METAL: &str = "metal";
pub const ASTEROID_1: &str = "asteroid_1";
pub const ASTEROID_2: &str = "asteroid_2";

#[derive(Clone)]
pub struct Tile {
    pub max: f64,
    pub min: f64,
    pub rand: f64,
    pub texture: Texture,
    pub id: String,
}

impl Tile {
    pub fn asteroid_1(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            texture: util::load_texture(&scene.display, include_bytes!("asteroid.png"))?,
            id: ASTEROID_1.to_string(),
        })
    }

    pub fn asteroid_2(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            texture: util::load_texture(&scene.display, include_bytes!("asteroid2.png"))?,
            id: ASTEROID_2.to_string(),
        })
    }

    pub fn metal(scene: &Scene) -> anyhow::Result<Self> {
        Ok(Self {
            max: 1.0,
            min: 2.0 / 3.0,
            rand: 2.0 / 3.0,
            texture: util::load_texture(&scene.display, include_bytes!("metal.png"))?,
            id: METAL.to_string(),
        })
    }

    pub fn space(scene: &Scene) -> anyhow::Result<Texture> {
        util::load_texture(&scene.display, include_bytes!("space.png"))
    }

    pub fn check(&self, rng: &mut StdRng, value: f64) -> Option<(&String, &Texture)> {
        if rng.gen_bool(self.rand) && self.max >= value && self.min <= value {
            Some((&self.id, &self.texture))
        } else {
            None
        }
    }
}
