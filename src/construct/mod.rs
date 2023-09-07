pub mod construct_data;
pub mod construct_manager;
pub mod item;
pub mod item_data;

pub use construct_data::ConstructData;
pub use construct_manager::ConstructManager;
pub use item::Item;
pub use item_data::ItemData;

use crate::{
    chunk::{Chunk, ChunkManager, Map, CHUNK_SIZE},
    player::State,
    tag::Tag,
    util,
};
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Context, Id,
    },
    id,
    math::{Mat3d, Vec2d},
};
use hex_instance::Instance;
use hex_physics::Physical;
use std::{f32::consts::PI, rc::Rc};

pub type UpdateFn<'a> =
    dyn Fn(Id, (&'a mut EntityManager, &'a mut ComponentManager)) -> anyhow::Result<()>;

pub const MINER: &str = "miner";
pub const RIGHT_ROUTER: &str = "right_router";
pub const LEFT_ROUTER: &str = "left_router";
pub const PICKUP_BIAS: f32 = 0.1;

#[derive(Clone)]
pub struct Construct<'a> {
    pub id: String,
    pub update: Rc<UpdateFn<'a>>,
    pub tick_amount: u32,
    pub update_tick: u32,
    pub eject_speed: f32,
}

impl Construct<'_> {
    pub fn miner(
        context: &Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Option<(Self, Instance)>> {
        let texture = util::load_texture(&context.display, include_bytes!("miner.png"))?;

        Ok(Tag::new("map")
            .find((em, cm))
            .and_then(|map| Some((map, Tag::new("player").find((em, cm))?)))
            .map(|(map, player)| {
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

                                        if let Some(state) = cm.get::<State>(player, em).cloned() {
                                            if let Some(tile_id) = tile_id {
                                                if let Some((item, instance)) =
                                                    state.items.get(tile_id)
                                                {
                                                    let entity = em.add();

                                                    cm.add(entity, instance.clone(), em);
                                                    cm.add(entity, item.clone(), em);
                                                    cm.add(entity, transform.clone(), em);
                                                    cm.add(
                                                        entity,
                                                        Physical::new(
                                                            (Mat3d::rotation(transform.rotation())
                                                                * (
                                                                    Vec2d::new(
                                                                        0.0,
                                                                        construct.eject_speed,
                                                                    ),
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
                            }

                            Ok(())
                        }),
                        tick_amount: 0,
                        update_tick: 100,
                        eject_speed: 1.0,
                    },
                    Instance::new(texture, [1.0; 4], -3.0, true),
                )
            }))
    }

    pub fn right_router(context: &Context) -> anyhow::Result<(Self, Instance)> {
        let texture = util::load_texture(&context.display, include_bytes!("right_router.png"))?;

        Ok((
            Self {
                id: RIGHT_ROUTER.to_string(),
                update: Rc::new(move |entity, (em, cm)| Self::router(entity, (em, cm), 1.0)),
                tick_amount: 0,
                update_tick: 1,
                eject_speed: 1.0,
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }

    pub fn left_router(context: &Context) -> anyhow::Result<(Self, Instance)> {
        let texture = util::load_texture(&context.display, include_bytes!("left_router.png"))?;

        Ok((
            Self {
                id: LEFT_ROUTER.to_string(),
                update: Rc::new(move |entity, (em, cm)| Self::router(entity, (em, cm), -1.0)),
                tick_amount: 0,
                update_tick: 1,
                eject_speed: 1.0,
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }

    fn router(
        entity: Id,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
        dir: f32,
    ) -> anyhow::Result<()> {
        if let Some((construct_position, construct_rotation)) = cm
            .get::<Transform>(entity, em)
            .map(|t| (t.position(), t.rotation()))
        {
            for e in em.entities.keys().cloned() {
                if let Some((iid, tid, force, position)) =
                    cm.get_id::<Item>(e, em).and_then(|iid| {
                        let item = cm.get_cache::<Item>(iid)?;

                        if item.last.map(|l| l != entity).unwrap_or(true) {
                            cm.get_id::<Transform>(e, em).and_then(|tid| {
                                Some((
                                    iid,
                                    tid,
                                    cm.get::<Physical>(e, em).map(|p| p.force)?,
                                    cm.get_cache::<Transform>(tid).map(|t| t.position())?,
                                ))
                            })
                        } else {
                            None
                        }
                    })
                {
                    let transformed = construct_position
                        + (Mat3d::rotation(construct_rotation)
                            * (Vec2d::new(0.0, -PICKUP_BIAS * 2.0), 1.0))
                            .0;
                    let direction = {
                        let direction =
                            (Mat3d::rotation(construct_rotation) * (Vec2d::new(0.0, 1.0), 1.0)).0;

                        Vec2d::new(direction.x().round(), direction.y().round()).normal()
                    };
                    let force = Vec2d::new(force.x().round(), force.y().round()).normal();

                    if direction == force
                        && (transformed.x() - position.x()).abs() <= PICKUP_BIAS
                        && (transformed.y() - position.y()).abs() <= PICKUP_BIAS
                    {
                        if let Some(transform) = cm.get_cache_mut::<Transform>(tid) {
                            transform.set_position(
                                (Mat3d::rotation(dir * (construct_rotation - PI / 2.0))
                                    * (Vec2d::new(0.0, PICKUP_BIAS * 2.0), 1.0))
                                    .0
                                    + construct_position,
                            );
                        }

                        if let Some(physical) = cm.get_mut::<Physical>(e, em) {
                            physical.force =
                                (Mat3d::rotation(dir * -PI / 2.0) * (physical.force, 1.0)).0;
                        }

                        if let Some(item) = cm.get_cache_mut::<Item>(iid) {
                            item.last = Some(e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
