use crate::construct::{ConstructData, ItemData};
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u32,
    pub player_position: [f32; 2],
    pub player_velocity: [f32; 2],
    pub constructs: Vec<ConstructData>,
    pub items: Vec<ItemData>,
}

impl SaveData {
    pub fn new(seed: u32, rng: &mut StdRng) -> Self {
        Self {
            seed,
            player_position: [Self::gen_map_coord(rng), Self::gen_map_coord(rng)],
            player_velocity: [0.0; 2],
            constructs: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn gen_map_coord(rng: &mut StdRng) -> f32 {
        rng.gen_range(0..1000) as f32
    }
}
