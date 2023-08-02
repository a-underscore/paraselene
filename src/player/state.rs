use super::SaveData;
use crate::{chunk::Tile, construct::Construct, construct::Item, SAVE_DIR};
use hex::{
    anyhow,
    assets::Texture,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Id, Scene,
    },
    id,
    once_cell::sync::Lazy,
};
use hex_instance::Instance;
use noise::Perlin;
use rand::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf};

pub static SAVE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(SAVE_DIR).join("map.json"));

#[derive(Clone)]
pub struct State<'a> {
    pub save_data: SaveData,
    pub rng: StdRng,
    pub perlin: Perlin,
    pub tiles: HashMap<String, Tile>,
    pub items: HashMap<String, (Item, Instance)>,
    pub constructs: HashMap<String, (Construct<'a>, Instance)>,
    pub space: Texture,
}

impl State<'_> {
    pub fn load(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
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
            tiles: vec![
                Tile::asteroid_1(scene)?,
                Tile::asteroid_2(scene)?,
                Tile::metal(scene)?,
            ]
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect(),
            items: vec![Item::metal(scene)?]
                .into_iter()
                .map(|ref i @ (ref item, _)| (item.id.clone(), i.clone()))
                .collect(),
            constructs: vec![Construct::miner(scene, (em, cm))?]
                .into_iter()
                .flatten()
                .map(|ref o @ (ref c, _)| (c.id.clone(), o.clone()))
                .collect(),
            space: Tile::space(scene)?,
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
