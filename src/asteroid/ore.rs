use crate::util;
use hex::{anyhow, assets::Texture, glium::Display};
use rand::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct Ore {
    pub max: f64,
    pub min: f64,
    pub rand: f64,
    pub texture: Vec<Texture>,
    pub id: Rc<String>,
}

impl Ore {
    pub fn rock(display: &Display) -> anyhow::Result<Self> {
        Ok(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            texture: vec![
                util::load_texture(display, include_bytes!("asteroid.png"))?,
                util::load_texture(display, include_bytes!("asteroid2.png"))?,
            ],
            id: Rc::new("rock".to_string()),
        })
    }

    pub fn metal(display: &Display) -> anyhow::Result<Self> {
        Ok(Self {
            max: 1.0,
            min: 2.0 / 3.0,
            rand: 2.0 / 3.0,
            texture: vec![util::load_texture(display, include_bytes!("metal.png"))?],
            id: Rc::new("metal".to_string()),
        })
    }

    pub fn check(&self, value: f64) -> Option<(&Rc<String>, &Texture)> {
        let mut rng = thread_rng();

        if rng.gen_bool(self.rand) && self.max >= value && self.min <= value {
            Some((&self.id, self.texture.choose(&mut rng)?))
        } else {
            None
        }
    }
}
