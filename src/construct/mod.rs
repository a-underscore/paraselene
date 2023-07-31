pub mod construct_data;
pub mod construct_manager;

pub use construct_data::ConstructData;
pub use construct_manager::ConstructManager;

use crate::{
    chunk::{Chunk, ChunkManager, Map},
    tag::Tag,
    util, CHUNK_SIZE,
};
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Id, Scene,
    },
    id,
};
use hex_instance::Instance;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

pub type UpdateFn<'a> =
    dyn Fn(Id, (&'a mut EntityManager, &'a mut ComponentManager)) -> anyhow::Result<()>;

#[derive(Clone)]
pub struct Construct<'a> {
    pub id: Rc<String>,
    pub update: Rc<UpdateFn<'a>>,
    pub time: Instant,
    pub update_duration: Duration,
}

impl Construct<'_> {
    pub fn miner(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Option<(Self, Instance)>> {
        let texture = util::load_texture(&scene.display, include_bytes!("miner.png"))?;

        Ok(Tag::new("map").find((em, cm)).map(|map| {
            (
                Self {
                    id: Rc::new("miner".to_string()),
                    update: Rc::new(move |e, (em, cm)| {
                        if let Some(transform) = cm.get::<Transform>(e, em).cloned() {
                            let pos = ChunkManager::chunk_pos(transform.position());

                            if let Some(id) = if let Some(map) = cm.get_mut::<Map>(map, em) {
                                map.loaded.get(&pos).cloned()
                            } else {
                                None
                            } {
                                if let Some(chunk) = cm.get::<Chunk>(id, em) {
                                    let x = CHUNK_SIZE as usize
                                        - ((pos.0 * CHUNK_SIZE) as usize
                                            - transform.position().x().floor() as usize);
                                    let y = CHUNK_SIZE as usize
                                        - ((pos.1 * CHUNK_SIZE) as usize
                                            - transform.position().y().floor() as usize);
                                    let tile = &chunk.grid.get(x).and_then(|c| c.get(y)?.clone());

                                    if let Some(tile) = tile {
                                        println!("{}", tile);
                                    }
                                }
                            }
                        }

                        Ok(())
                    }),
                    time: Instant::now(),
                    update_duration: Duration::from_millis(500),
                },
                Instance::new(texture, [1.0; 4], -3.0, true),
            )
        }))
    }
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
