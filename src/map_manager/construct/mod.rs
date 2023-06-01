pub mod construct_manager;

pub use construct_manager::ConstructManager;

use hex::{
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
        Id,
    },
    id,
};
use std::rc::Rc;

pub type UpdateFn<'a> = dyn Fn(Id, (&'a EntityManager, &'a ComponentManager));

#[derive(Clone)]
pub struct Construct<'a> {
    pub update: Rc<UpdateFn<'a>>,
    pub active: bool,
}

impl Component for Construct<'_> {
    fn id() -> Id {
        id!()
    }
}
