pub mod construct_data;
pub mod construct_manager;
pub mod item;

pub use construct_data::ConstructData;
pub use construct_manager::ConstructManager;
pub use item::Item;

use crate::{
    chunk::{tile::METAL, Chunk, ChunkManager, Map, CHUNK_SIZE},
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
    math::{Mat3d, Vec2d},
};
use hex_instance::Instance;
use hex_physics::Physical;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

pub type UpdateFn<'a> =
    dyn Fn(Id, (&'a mut EntityManager, &'a mut ComponentManager)) -> anyhow::Result<()>;

pub const MINER: &str = "miner";

#[derive(Clone)]
pub struct Construct<'a> {
    pub id: String,
    pub update: Rc<UpdateFn<'a>>,
    pub time: Instant,
    pub update_duration: Duration,
    pub eject_speed: f32,
}

impl Construct<'_> {
    pub fn miner(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Option<(Self, Instance)>> {
        let texture = util::load_texture(&scene.display, include_bytes!("miner.png"))?;
        let tile = Instance::new(
            util::load_texture(&scene.display, include_bytes!("metal.png"))?,
            [1.0; 4],
            -3.5,
            true,
        );

        Ok(Tag::new("map").find((em, cm)).map(|map| {
            (
                Self {
                    id: MINER.to_string(),
                    update: Rc::new(move |e, (em, cm)| {
                        if let Some((transform, construct)) = cm
                            .get::<Transform>(e, em)
                            .cloned()
                            .and_then(|t| Some((t, cm.get::<Construct>(e, em).cloned()?)))
                        {
                            let pos = ChunkManager::chunk_pos(transform.position());

                            if let Some(id) = if let Some(map) = cm.get_mut::<Map>(map, em) {
                                map.loaded.get(&pos).cloned()
                            } else {
                                None
                            } {
                                if let Some(chunk) = cm.get::<Chunk>(id, em).cloned() {
                                    let x = CHUNK_SIZE as usize
                                        - ((pos.0 * CHUNK_SIZE) as usize
                                            - transform.position().x().floor() as usize);
                                    let y = CHUNK_SIZE as usize
                                        - ((pos.1 * CHUNK_SIZE) as usize
                                            - transform.position().y().floor() as usize);
                                    let tile_id =
                                        &chunk.grid.get(x).and_then(|c| c.get(y)?.clone());

                                    if let Some(tile_id) = tile_id {
                                        if tile_id == METAL {
                                            let entity = em.add();

                                            cm.add(entity, tile.clone(), em);
                                            cm.add(entity, transform.clone(), em);
                                            cm.add(
                                                entity,
                                                Physical::new(
                                                    (Mat3d::rotation(transform.rotation())
                                                        * (
                                                            Vec2d::new(0.0, construct.eject_speed),
                                                            1.0,
                                                        ))
                                                        .0,
                                                    true,
                                                ),
                                                em,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        Ok(())
                    }),
                    time: Instant::now(),
                    update_duration: Duration::from_millis(1000),
                    eject_speed: 1.0,
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
