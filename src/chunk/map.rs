use hex::{
    ecs::{component_manager::Component, Id},
    id,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Map {
    pub load_queue: Vec<((u32, u32), bool)>,
    pub loaded: HashMap<(u32, u32), (bool, Id)>,
}

impl Map {
    pub fn queue_load(&mut self, queue @ (chunk, _): ((u32, u32), bool)) {
        let chunks: Vec<_> = self.load_queue.iter().cloned().map(|(c, _)| c).collect();

        if !(chunks.contains(&chunk) || self.loaded.contains_key(&chunk)) {
            self.load_queue.push(queue);
        }
    }
}

impl Component for Map {
    fn id() -> Id {
        id!()
    }
}
