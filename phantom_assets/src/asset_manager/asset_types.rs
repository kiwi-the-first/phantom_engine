use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum AssetType {
    Sprite,
    Audio,
    Invalid,
}
