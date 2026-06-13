use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset_manager::AssetType;

#[derive(Serialize, Deserialize)]
pub struct PhantomAsset {
    id: Uuid,
    asset_type: AssetType,
    asset_path: PathBuf,
}

impl PhantomAsset {
    pub fn new(id: Uuid, asset_type: AssetType, asset_path: PathBuf) -> Self {
        Self {
            id,
            asset_type,
            asset_path,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_asset_type(&self) -> AssetType {
        self.asset_type
    }

    pub fn get_asset_path(&self) -> PathBuf {
        self.asset_path.clone()
    }

    pub fn set_id(&mut self, uuid: Uuid) {
        self.id = uuid;
    }

    pub fn set_asset_type(&mut self, asset_type: AssetType) {
        self.asset_type = asset_type
    }

    pub fn set_asset_path(&mut self, asset_path: PathBuf) {
        self.asset_path = asset_path;
    }
}
