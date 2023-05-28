use super::SaveData;
use crate::SAVE_DIR;
use hex::{
    anyhow,
    ecs::{component_manager::Component, Id},
    id,
    once_cell::sync::Lazy,
};
use noise::Perlin;
use rand::prelude::*;
use std::{fs, path::PathBuf};

pub static SAVE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(SAVE_DIR).join("map.json"));

pub struct State {
    pub save_data: SaveData,
    pub rng: StdRng,
    pub perlin: Perlin,
}

impl State {
    pub fn load() -> anyhow::Result<Self> {
        let (mut rng, save_data) = fs::read_to_string(&*SAVE_PATH)
            .ok()
            .map(|s| -> anyhow::Result<_> {
                let save_data: SaveData = serde_json::from_str(&s)?;

                Ok((StdRng::seed_from_u64(save_data.seed), save_data))
            })
            .unwrap_or_else(|| {
                let seed = thread_rng().gen_range(u64::MIN..u64::MAX);
                let mut rng = StdRng::seed_from_u64(seed);
                let data = SaveData::new(seed, &mut rng);
                let content = serde_json::to_string(&data)?;

                fs::write(&*SAVE_PATH, content)?;

                Ok((rng, data))
            })?;
        let perlin = Perlin::new(rng.gen_range(u32::MIN..u32::MAX));

        Ok(Self {
            save_data,
            perlin,
            rng,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string(&self.save_data)?;
        fs::write(&*SAVE_PATH, content)?;

        Ok(())
    }
}

impl Component for State {
    fn id() -> Id {
        id!()
    }
}
