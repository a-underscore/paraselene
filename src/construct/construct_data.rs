use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstructData {
    pub id: String,
    pub position: [f32; 2],
    pub rotation: f32,
    pub tick_amount: u32,
}
