use std::{
    fs,
    path::{Path, PathBuf},
};

use phantom_common::dirs;
use phantom_core::ecs::{World, components::Sprite};
use phantom_project::project_manager::project_manager::ProjectManager;

pub struct BuildSystem;

impl BuildSystem {
    pub fn build(project_path: PathBuf) -> anyhow::Result<()> {
        let (project, mut world) = ProjectManager::load(project_path.clone())?;

        BuildSystem::create_build_dirs(&project_path)?;

        let asset_paths = BuildSystem::collect_sprite_assets(&world);
        BuildSystem::copy_assets(&asset_paths, &project_path)?;
        BuildSystem::update_sprite_paths(&mut world)?;
        let world_bytes = world.serialize();
        fs::write(
            dirs::BuildDirs::data(&project_path).join(project.entry_world.file_name().unwrap()),
            world_bytes,
        )?;

        Ok(())
    }
    fn create_build_dirs(build_path: &PathBuf) -> anyhow::Result<()> {
        let assets_path = dirs::BuildDirs::assets(&build_path);
        log::trace!("Creating directories {}", assets_path.to_string_lossy());
        std::fs::create_dir_all(assets_path)?;
        Ok(())
    }
    fn collect_sprite_assets(world: &World) -> Vec<String> {
        world
            .query_with::<Sprite>()
            .iter()
            .map(|entity| {
                world
                    .get_component::<Sprite>(*entity)
                    .unwrap()
                    .asset_path
                    .clone()
            })
            .filter(|path| !path.is_empty())
            .collect()
    }
    fn copy_assets(assets: &[String], project_path: &PathBuf) -> anyhow::Result<()> {
        for asset_path in assets {
            let source = project_path.join(asset_path);
            let file_name = source
                .file_name()
                .ok_or(anyhow::anyhow!("invalid asset path: {}", asset_path))?;
            let dest = dirs::BuildDirs::assets(&project_path).join(file_name);
            std::fs::copy(&source, &dest)?;
            log::trace!(
                "Copied {} to {}",
                source.to_string_lossy(),
                dest.to_string_lossy()
            );
        }
        Ok(())
    }
    fn update_sprite_paths(world: &mut World) -> anyhow::Result<()> {
        let entities = world.query_with::<Sprite>();
        for entity in entities {
            let sprite = world.get_component_mut::<Sprite>(entity).unwrap();
            if !sprite.asset_path.is_empty() {
                let file_name = PathBuf::from(&sprite.asset_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                sprite.asset_path = format!("assets/{}", file_name);
            }
        }
        Ok(())
    }
}
