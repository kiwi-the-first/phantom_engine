use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Ok, Result};
use phantom_core::ecs::{Entity, World, components::Sprite};

use crate::{asset_manager::AssetManager, texture_loader::asset_types::texture::Texture};

#[derive(Default)]
pub struct TextureLoader {
    processed_entities: HashMap<u32, String>,
    pub textures: HashMap<PathBuf, Texture>,
}

impl TextureLoader {
    pub fn load_sprite_assets(
        &mut self,
        asset_manager: &mut AssetManager,
        asset_root: &Path,
    ) -> Result<()> {
        let sprites = asset_manager.grab_all_registered_sprite_assets();

        for sprite in sprites {
            let path = sprite.get_asset_path();
            if !self.textures.contains_key(&path) {
                let sprite_path = if path.is_absolute() {
                    path.clone()
                } else {
                    asset_root.join(&path)
                };
                let sprite_bytes = std::fs::read(&sprite_path)?;
                let sprite_image = image::load_from_memory(&sprite_bytes)?;
                let sprite_rgba = sprite_image.to_rgba8();
                let texture = Texture {
                    rgba_image: sprite_rgba,
                };
                self.textures.insert(path.clone(), texture);
            }
        }

        // // Early exit: if all sprite entities have been processed, skip
        // if !entitys_with_sprites.is_empty()
        //     && entitys_with_sprites.iter().all(|e| {
        //         let sprite_component = world.get_component::<Sprite>(*e).unwrap();
        //         self.processed_entities.get(&e.id) == Some(&sprite_component.asset)
        //     })
        // {
        //     return Ok(());
        // }
        // for entity in entitys_with_sprites {
        //     let sprite_component = world.get_component::<Sprite>(entity).unwrap();
        //     let sprite_path = sprite_component.asset.clone();

        //     let full_path = if sprite_path.starts_with(project_root.to_str().unwrap()) {
        //         PathBuf::from(&sprite_path)
        //     } else {
        //         project_root.join(&sprite_path)
        //     };

        //     if sprite_path.is_empty() || !Path::new(&full_path).exists() {
        //         log::warn!("Invalid or missing sprite path: {}", sprite_path);
        //         self.processed_entities
        //             .insert(entity.id, sprite_path.clone());
        //         continue;
        //     }

        //     if self.textures.contains_key(&sprite_path) {
        //         self.processed_entities
        //             .insert(entity.id, sprite_path.clone());
        //         continue;
        //     }

        //     let sprite_bytes = std::fs::read(full_path.clone())?;
        //     let sprite_image = image::load_from_memory(&sprite_bytes)?;
        //     let sprite_rgba = sprite_image.to_rgba8();
        //     let texture = Texture {
        //         rgba_image: sprite_rgba,
        //     };
        //     self.textures.insert(sprite_path.clone(), texture);
        //     self.processed_entities
        //         .insert(entity.id, sprite_path.clone());
        //     log::trace!("Asset {} Loaded!", sprite_path);
        //}

        Ok(())
    }
}
