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
    math::{Mat3d, Vec2d},
};
use hex_instance::Instance;
use hex_physics::Physical;
use std::{f32::consts::PI, rc::Rc};

pub type UpdateFn = dyn Fn(Id, (&mut EntityManager, &mut ComponentManager)) -> anyhow::Result<()>;

pub const MINER: &str = "miner";
pub const RIGHT_ROUTER: &str = "right_router";
pub const LEFT_ROUTER: &str = "left_router";
pub const RIGHT_SPLITTER: &str = "right_splitter";
pub const LEFT_SPLITTER: &str = "left_splitter";
pub const FURNACE: &str = "furnace";
pub const PICKUP_BIAS: f32 = 0.1;

#[derive(Clone)]
pub struct Construct {
    pub id: String,
    pub update: Rc<UpdateFn>,
    pub tick_amount: u32,
    pub update_tick: u32,
    pub mode: Option<bool>,
}

impl Construct {
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
                            if let Some(transform) = cm.get::<Transform>(e).cloned() {
                                let pos = ChunkManager::chunk_pos(transform.position());

                                if let Some(id) = if let Some(map) = cm.get_mut::<Map>(map) {
                                    map.loaded.get(&pos).cloned()
                                } else {
                                    None
                                } {
                                    if let Some(chunk) = cm.get::<Chunk>(id).cloned() {
                                        let x = CHUNK_SIZE as usize
                                            - ((pos.0 * CHUNK_SIZE) as usize
                                                - transform.position().x().floor() as usize);
                                        let y = CHUNK_SIZE as usize
                                            - ((pos.1 * CHUNK_SIZE) as usize
                                                - transform.position().y().floor() as usize);
                                        let tile_id =
                                            &chunk.grid.get(x).and_then(|c| c.get(y)?.clone());

                                        if let Some(state) = cm.get::<State>(player).cloned() {
                                            if let Some(tile_id) = tile_id {
                                                if let Some((item, instance)) =
                                                    state.items.get(tile_id)
                                                {
                                                    let entity = em.add();

                                                    cm.add(entity, instance.clone(), em);
                                                    cm.add(entity, item.clone(), em);
                                                    cm.add(
                                                        entity,
                                                        Transform::new(
                                                            transform.position(),
                                                            0.0,
                                                            Vec2d([1.0; 2]),
                                                            true,
                                                        ),
                                                        em,
                                                    );
                                                    cm.add(
                                                        entity,
                                                        Physical::new(
                                                            (Mat3d::rotation(transform.rotation())
                                                                * (Vec2d::new(0.0, 1.0), 1.0))
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
                        update_tick: 1000,
                        mode: None,
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
                mode: None,
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
                mode: None,
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }

    fn router(
        entity: Id,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
        dir: f32,
    ) -> anyhow::Result<()> {
        if let Some(construct_transform) = cm.get::<Transform>(entity).cloned() {
            for e in em.entities() {
                if let Some((force, item_position)) = cm.get::<Item>(e).and_then(|item| {
                    if item.last.map(|l| l != entity).unwrap_or(true) {
                        Some((
                            cm.get::<Physical>(e).map(|p| p.force)?,
                            cm.get::<Transform>(e).map(|t| t.position())?,
                        ))
                    } else {
                        None
                    }
                }) {
                    if Self::pickup(&construct_transform, item_position, force) {
                        if let Some(transform) = cm.get_mut::<Transform>(e) {
                            transform.set_position(
                                (Mat3d::rotation(construct_transform.rotation() + dir * -PI / 2.0)
                                    * (Vec2d::new(0.0, PICKUP_BIAS * 2.0), 1.0))
                                    .0
                                    + construct_transform.position(),
                            );
                        }

                        if let Some(physical) = cm.get_mut::<Physical>(e) {
                            physical.force =
                                (Mat3d::rotation(dir * -PI / 2.0) * (physical.force, 1.0)).0;
                        }

                        if let Some(item) = cm.get_mut::<Item>(e) {
                            item.last = Some(entity);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn left_splitter(context: &Context) -> anyhow::Result<(Self, Instance)> {
        let texture = util::load_texture(&context.display, include_bytes!("left_splitter.png"))?;

        Ok((
            Self {
                id: LEFT_SPLITTER.to_string(),
                update: Rc::new(move |entity, (em, cm)| Self::splitter(entity, (em, cm), -1.0)),
                tick_amount: 0,
                update_tick: 1,
                mode: Some(true),
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }

    pub fn right_splitter(context: &Context) -> anyhow::Result<(Self, Instance)> {
        let texture = util::load_texture(&context.display, include_bytes!("right_splitter.png"))?;

        Ok((
            Self {
                id: RIGHT_SPLITTER.to_string(),
                update: Rc::new(move |entity, (em, cm)| Self::splitter(entity, (em, cm), 1.0)),
                tick_amount: 0,
                update_tick: 1,
                mode: Some(true),
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }

    fn splitter(
        entity: Id,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
        dir: f32,
    ) -> anyhow::Result<()> {
        if let Some(construct_transform) = cm.get::<Transform>(entity).cloned() {
            for e in em.entities() {
                if let Some((force, item_position)) = cm.get::<Item>(e).and_then(|item| {
                    if item.last.map(|l| l != entity).unwrap_or(true) {
                        Some((
                            cm.get::<Physical>(e).map(|p| p.force)?,
                            cm.get::<Transform>(e).map(|t| t.position())?,
                        ))
                    } else {
                        None
                    }
                }) {
                    if Self::pickup(&construct_transform, item_position, force) {
                        if let Some(m) = cm.get_mut::<Construct>(entity).and_then(|c| {
                            if let Some(m) = &mut c.mode {
                                *m = !*m;

                                Some(*m)
                            } else {
                                None
                            }
                        }) {
                            if m {
                                if let Some(transform) = cm.get_mut::<Transform>(e) {
                                    transform.set_position(
                                        (Mat3d::rotation(
                                            construct_transform.rotation() + dir * -PI / 2.0,
                                        ) * (Vec2d::new(0.0, PICKUP_BIAS * 2.0), 1.0))
                                            .0
                                            + construct_transform.position(),
                                    );
                                }

                                if let Some(physical) = cm.get_mut::<Physical>(e) {
                                    physical.force = (Mat3d::rotation(dir * -PI / 2.0)
                                        * (physical.force, 1.0))
                                        .0;
                                }
                            }

                            if let Some(item) = cm.get_mut::<Item>(e) {
                                item.last = Some(entity);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn furnace(
        context: &Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Option<(Self, Instance)>> {
        let texture = util::load_texture(&context.display, include_bytes!("furnace.png"))?;

        Ok(Tag::new("player").find((em, cm)).map(|player| {
            (
                Self {
                    id: FURNACE.to_string(),
                    update: Rc::new(move |entity, (em, cm)| {
                        if let Some(transform) = cm.get::<Transform>(entity).cloned() {
                            for e in em.entities() {
                                if let Some((force, position, refined)) =
                                    cm.get::<Item>(e).and_then(|item| {
                                        if item.last.map(|l| l != entity).unwrap_or(true) {
                                            Some((
                                                cm.get::<Physical>(e).map(|p| p.force)?,
                                                cm.get::<Transform>(e).map(|t| t.position())?,
                                                item.refined.clone(),
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                {
                                    if Self::pickup(&transform, position, force) {
                                        if let Some(item_id) = refined {
                                            if let Some((new_item, new_instance)) =
                                                if let Some(state) = cm.get::<State>(player) {
                                                    state.items.get(&item_id).cloned()
                                                } else {
                                                    None
                                                }
                                            {
                                                if let Some(item) = cm.get_mut::<Item>(e) {
                                                    *item = new_item;
                                                    item.last = Some(entity);
                                                }

                                                if let Some(instance) = cm.get_mut::<Instance>(e) {
                                                    *instance = new_instance;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Ok(())
                    }),
                    tick_amount: 0,
                    update_tick: 1,
                    mode: None,
                },
                Instance::new(texture, [1.0; 4], -3.0, true),
            )
        }))
    }

    fn pickup(construct_transform: &Transform, item_position: Vec2d, force: Vec2d) -> bool {
        let transformed = construct_transform.position()
            + (Mat3d::rotation(construct_transform.rotation())
                * (Vec2d::new(0.0, -PICKUP_BIAS * 2.0), 1.0))
                .0;
        let direction = {
            let direction =
                (Mat3d::rotation(construct_transform.rotation()) * (Vec2d::new(0.0, 1.0), 1.0)).0;

            Vec2d::new(direction.x().round(), direction.y().round()).normal()
        };
        let force = Vec2d::new(force.x().round(), force.y().round()).normal();

        direction == force
            && (transformed.x() - item_position.x()).abs() <= PICKUP_BIAS
            && (transformed.y() - item_position.y()).abs() <= PICKUP_BIAS
    }
}

impl Component for Construct {}
