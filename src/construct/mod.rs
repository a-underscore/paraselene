pub mod construct_data;
pub mod construct_manager;

pub use construct_data::ConstructData;
pub use construct_manager::ConstructManager;

use crate::{
    chunk::{Chunk, ChunkManager, Map},
    tag::Tag,
    util,
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
use std::rc::Rc;

pub type UpdateFn<'a> =
    dyn Fn(Id, (&'a mut EntityManager, &'a mut ComponentManager)) -> anyhow::Result<()>;

#[derive(Clone)]
pub struct Construct<'a> {
    pub id: Rc<String>,
    pub update: Rc<UpdateFn<'a>>,
}

impl Construct<'_> {
    pub fn miner(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Option<(Self, Instance)>> {
        let texture = util::load_texture(&scene.display, include_bytes!("miner.png"))?;

        Ok(if let Some(map) = Tag::new("map").find((em, cm)) {
            Some((
                Self {
                    id: Rc::new("miner".to_string()),
                    update: Rc::new(move |e, (em, cm)| {
                        if let Some(transform) = cm.get::<Transform>(e, em).cloned() {
                            let pos = ChunkManager::chunk_pos(transform.position());

                            if let Some(id) = if let Some(map) = cm.get_mut::<Map>(map, em) {
                                map.queue_load((pos, true));

                                map.loaded.get(&pos).map(|(_, c)| *c)
                            } else {
                                None
                            } {
                                if let Some(chunk) = cm.get::<Chunk>(id, em) {
                                    let _ = &chunk.grid[pos.0 as usize
                                        + transform.position().x().floor() as usize]
                                        [pos.1 as usize
                                            + transform.position().y().floor() as usize];
                                }
                            }
                        }

                        Ok(())
                    }),
                },
                Instance::new(texture, [1.0; 4], -3.0, true),
            ))
        } else {
            None
        })
    }
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
