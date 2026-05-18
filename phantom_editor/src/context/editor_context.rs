use std::path::PathBuf;

use anyhow::Result;
use phantom_core::ecs::{Entity, World};
use phantom_project::phantom_project::PhantomProject;
use phantom_runtime::asset_manager::asset_manager::{self, AssetManager};

pub struct EditorContext {
    pub project_path: PathBuf,
    pub project: PhantomProject,
    pub active_world: World,
    pub selected_entity: Option<Entity>,
    pub asset_manager: AssetManager,
}

impl EditorContext {
    pub fn new(project_path: PathBuf, project: PhantomProject, world: World) -> Self {
        let asset_manager = AssetManager::new();
        Self {
            project_path: project_path,
            project: project,
            active_world: world,
            selected_entity: None,
            asset_manager,
        }
    }

    pub fn sync_assets(&mut self) -> Result<()> {
        self.asset_manager
            .load_sprite_assets(&self.active_world, self.project_path.as_path())?;
        Ok(())
    }
}
