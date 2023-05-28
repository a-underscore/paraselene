use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u64,
    pub player_position: [f32; 2],
}

impl SaveData {
    pub fn new(seed: u64, rng: &mut StdRng) -> Self {
        let x = Self::gen_map_coord(rng);
        let y = Self::gen_map_coord(rng);

        Self {
            seed,
            player_position: [x, y],
        }
    }

    pub fn gen_map_coord(rng: &mut StdRng) -> f32 {
        rng.gen_range(0..1000) as f32
    }
}
