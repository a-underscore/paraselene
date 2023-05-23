use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

pub struct Chunk {
    pub active: bool,
}

impl Chunk {
    pub fn new(active: bool) -> Self {
        Self { active }
    }
}

impl Component for Chunk {
    fn id() -> Id {
        id!()
    }
}
