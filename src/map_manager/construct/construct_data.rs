use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConstructData {
    pub position: (u64, u64),
    pub id: String,
}
