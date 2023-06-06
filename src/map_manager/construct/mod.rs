pub mod construct_data;
pub mod construct_manager;

pub use construct_data::ConstructData;
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
use hex_instance::Instance;
use std::rc::Rc;

pub type UpdateFn<'a> = dyn Fn(Id, (&'a mut EntityManager, &'a mut ComponentManager));

#[derive(Clone)]
pub struct Construct<'a> {
    pub id: Rc<String>,
    pub update: Rc<UpdateFn<'a>>,
}

impl<'a> Construct<'a> {
    pub fn miner(scene: &Scene) -> anyhow::Result<(Self, Instance, Sprite)> {
        let texture = util::load_texture(&scene.display, include_bytes!("miner.png"))?;

        Ok((
            Self {
                id: Rc::new("miner".to_string()),
                update: Rc::new(|_, _| {
                    println!("I am here, I am a construct, and my update method is being called")
                }),
            },
            Instance::new(texture.clone(), [1.0; 4], -3.0, true),
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                texture,
                [1.0; 4],
                0.0,
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
