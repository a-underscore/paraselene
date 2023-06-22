use hex::{
    ecs::{component_manager::Component, Id},
    id,
};

#[derive(Copy, Clone)]
pub struct Trail(pub usize);

impl Component for Trail {
    fn id() -> Id {
        id!()
    }
}
