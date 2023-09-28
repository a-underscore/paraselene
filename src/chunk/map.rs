use hex::{
    ecs::{component_manager::Component, Id},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Map {
    pub load_queue: Vec<(u32, u32)>,
    pub loaded: HashMap<(u32, u32), Id>,
}

impl Map {
    pub fn queue_load(&mut self, chunk: (u32, u32)) {
        if !(self.load_queue.contains(&chunk) || self.loaded.contains_key(&chunk)) {
            self.load_queue.push(chunk);
        }
    }
}

impl Component for Map {
}
