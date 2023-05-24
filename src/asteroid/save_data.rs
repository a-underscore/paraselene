use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u64,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            seed: thread_rng().gen_range(u64::MIN..u64::MAX),
        }
    }
}
