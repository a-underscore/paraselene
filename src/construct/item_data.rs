use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub id: String,
}
