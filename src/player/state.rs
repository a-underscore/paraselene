use super::SaveData;
use crate::{chunk::Ore, construct::Construct, SAVE_DIR};
use hex::{
    anyhow,
    assets::Texture,
    components::Sprite,
    ecs::Scene,
    ecs::{component_manager::Component, Id},
    id,
    once_cell::sync::Lazy,
};
use hex_instance::Instance;
use noise::Perlin;
use rand::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf};

pub static SAVE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(SAVE_DIR).join("map.json"));

pub struct State<'a> {
    pub save_data: SaveData,
    pub rng: StdRng,
    pub perlin: Perlin,
    pub ores: HashMap<String, Ore>,
    pub constructs: HashMap<String, (Construct<'a>, Instance, Sprite)>,
    pub space: Texture,
    pub placed: HashMap<(u64, u64), (String, Id)>,
}

impl State<'_> {
    pub fn load(scene: &Scene) -> anyhow::Result<Self> {
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

                Ok((rng, data))
            })?;
        let perlin = Perlin::new(rng.gen_range(u32::MIN..u32::MAX));

        Ok(Self {
            save_data,
            perlin,
            rng,
            ores: vec![
                Ore::asteroid_1(scene)?,
                Ore::asteroid_2(scene)?,
                Ore::metal(scene)?,
            ]
            .into_iter()
            .map(|o| (o.id.as_ref().clone(), o))
            .collect(),
            constructs: vec![Construct::miner(scene)?]
                .into_iter()
                .map(|ref o @ (ref c, _, _)| (c.id.as_ref().clone(), o.clone()))
                .collect(),
            space: Ore::space(scene)?,
            placed: HashMap::new(),
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string(&self.save_data)?;

        fs::write(&*SAVE_PATH, content)?;

        Ok(())
    }
}

impl Component for State<'_> {
    fn id() -> Id {
        id!()
    }
}
