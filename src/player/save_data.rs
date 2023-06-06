use crate::map_manager::construct::ConstructData;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u64,
    pub player_position: [f32; 2],
    pub constructs: Vec<ConstructData>,
}

impl SaveData {
    pub fn new(seed: u64, rng: &mut StdRng) -> Self {
        Self {
            seed,
            player_position: [Self::gen_map_coord(rng), Self::gen_map_coord(rng)],
            constructs: Vec::new(),
        }
    }

    pub fn gen_map_coord(rng: &mut StdRng) -> f32 {
        rng.gen_range(0..1000) as f32
    }
}
