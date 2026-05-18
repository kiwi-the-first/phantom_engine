use std::collections::HashMap;

use crate::asset_manager::asset_types::texture::Texture;

pub struct AssetManager {
    textures: HashMap<String, Texture>,
}
