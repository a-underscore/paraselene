use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TileData {
    pub position: [f32; 2],
    pub id: Option<String>,
}
