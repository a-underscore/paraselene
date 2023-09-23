use super::SaveData;
use crate::{chunk::Tile, construct::Construct, construct::Item, SAVE_DIR};
use hex::{
    anyhow,
    assets::Texture,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Context, Id,
    },
    id,
    once_cell::sync::Lazy,
};
use hex_instance::Instance;
use noise::Perlin;
use rand::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf};

pub static SAVE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(SAVE_DIR).join("map.json"));

pub const MENU_MODE: u32 = 0;
pub const GAME_MODE: u32 = 1;

#[derive(Clone)]
pub struct State {
    pub save_data: SaveData,
    pub rng: StdRng,
    pub perlin: Perlin,
    pub tiles: HashMap<String, Tile>,
    pub items: HashMap<String, (Item, Instance)>,
    pub constructs: HashMap<String, (Construct, Instance)>,
    pub space: Texture,
    pub mode: u32,
}

impl State {
    pub fn load(
        context: &Context,
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
                Tile::asteroid_1(context)?,
                Tile::asteroid_2(context)?,
                Tile::metal(context)?,
            ]
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect(),
            items: vec![Item::metal(context)?, Item::refined_metal(context)?]
                .into_iter()
                .map(|ref i @ (ref item, _)| (item.id.clone(), i.clone()))
                .collect(),
            constructs: vec![
                Construct::miner(context, (em, cm))?,
                Construct::furnace(context, (em, cm))?,
            ]
            .into_iter()
            .flatten()
            .chain(vec![
                Construct::left_router(context)?,
                Construct::right_router(context)?,
                Construct::left_splitter(context)?,
                Construct::right_splitter(context)?,
            ])
            .map(|ref o @ (ref c, _)| (c.id.clone(), o.clone()))
            .collect(),
            space: Tile::space(context)?,
            mode: MENU_MODE,
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
