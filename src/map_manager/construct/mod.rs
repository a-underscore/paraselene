pub mod construct_manager;

pub use construct_manager::ConstructManager;

use crate::util;
use hex::{
    anyhow,
    assets::Shape,
    components::Sprite,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Id, Scene,
    },
    id,
    math::Vec2d,
};
use std::rc::Rc;

pub type UpdateFn<'a> = dyn Fn(Id, (&'a EntityManager, &'a ComponentManager));

#[derive(Clone)]
pub struct Construct<'a> {
    pub update: Rc<UpdateFn<'a>>,
}

impl<'a> Construct<'a> {
    pub fn miner(scene: &Scene) -> anyhow::Result<(Self, Sprite)> {
        Ok((
            Self {
                update: Rc::new(|_, _| println!("miner test")),
            },
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                util::load_texture(&scene.display, include_bytes!("miner.png"))?,
                [1.0; 4],
                -5.0,
                true,
            ),
        ))
    }
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
