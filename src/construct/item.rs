use crate::{chunk::tile::METAL, util};
use hex::{
    anyhow,
    ecs::{component_manager::Component, Context, Id},
    id,
};
use hex_instance::Instance;

#[derive(Clone)]
pub struct Item {
    pub id: String,
    pub last: Option<Id>,
}

impl Item {
    pub fn metal(context: &Context) -> anyhow::Result<(Self, Instance)> {
        Ok((
            Self {
                id: METAL.to_string(),
                last: None,
            },
            Instance::new(
                util::load_texture(&context.display, include_bytes!("metal.png"))?,
                [1.0; 4],
                -3.5,
                true,
            ),
        ))
    }
}

impl Component for Item {
    fn id() -> Id {
        id!()
    }
}
