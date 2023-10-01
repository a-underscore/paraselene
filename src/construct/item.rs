use crate::util;
use hex::{
    anyhow,
    ecs::{component_manager::Component, Context, Id},
};
use hex_instance::Instance;

pub const REFINED_METAL: &str = "refined_metal";
pub const METAL: &str = "metal";

#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub last: Option<Id>,
    pub refined: Option<String>,
}

impl Item {
    pub fn metal(context: &Context) -> anyhow::Result<(Self, Instance)> {
        Ok((
            Self {
                id: METAL.to_string(),
                last: None,
                refined: Some(REFINED_METAL.to_string()),
            },
            Instance::new(
                util::load_texture(&context.display, include_bytes!("metal.png"))?,
                [1.0; 4],
                -3.5,
                true,
            ),
        ))
    }

    pub fn refined_metal(context: &Context) -> anyhow::Result<(Self, Instance)> {
        Ok((
            Self {
                id: REFINED_METAL.to_string(),
                last: None,
                refined: None,
            },
            Instance::new(
                util::load_texture(&context.display, include_bytes!("refined_metal.png"))?,
                [1.0; 4],
                -3.5,
                true,
            ),
        ))
    }
}

impl Component for Item {}
