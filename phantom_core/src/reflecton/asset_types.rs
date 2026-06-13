use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct SpriteAsset(pub Uuid);
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct AudioAsset(pub Uuid);
