pub mod construct_data;
pub mod construct_manager;

pub use construct_data::ConstructData;
pub use construct_manager::ConstructManager;

use crate::{chunk::Map, tag::Tag, util};
use hex::{
    anyhow,
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
    pub fn miner(scene: &Scene) -> anyhow::Result<(Self, Instance)> {
        let texture = util::load_texture(&scene.display, include_bytes!("miner.png"))?;

        Ok((
            Self {
                id: Rc::new("miner".to_string()),
                update: Rc::new(|_, (em, cm)| {
                    if let Some(map) = Tag::new("map").find((em, cm)) {
                        if let Some(_) = cm.get::<Map>(map, em) {}
                    }

                    Ok(())
                }),
            },
            Instance::new(texture, [1.0; 4], -3.0, true),
        ))
    }
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
