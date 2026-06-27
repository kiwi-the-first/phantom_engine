use std::io::Write;
use std::path::PathBuf;

use crate::phantom_project::PhantomProject;
use anyhow::{Ok, Result};
use phantom_core::ecs::World;

pub struct ProjectManager {}

impl ProjectManager {
    /// Loads a phantom project and its entry world.
    ///
    /// # Arguments
    /// * `path` - Path to the **project root directory** containing the `.pproject` file
    pub fn load(path: PathBuf) -> Result<(PhantomProject, World)> {
        let pproject = ProjectManager::get_pproject(&path)?;
        let world_path = path.join(&pproject.entry_world);
        let world_bytes = std::fs::read(&world_path)?;
        let world = World::deserialize(&world_bytes);
        Ok((pproject, world))
    }

    /// Saves the world to the project's entry world file.
    ///
    /// # Arguments
    /// * `path` - Path to the **project root directory** containing the `.pproject` file
    /// * `world` - The world to serialize and save
    pub fn save(path: PathBuf, world: &World) -> anyhow::Result<()> {
        let pproject = ProjectManager::get_pproject(&path)?;
        let world_path = path.join(&pproject.entry_world);
        let world_data = world.serialize();
        let mut file = std::fs::File::create(&world_path)?;
        file.write_all(&world_data)?;
        Ok(())
    }
    /// Finds and deserializes the `.pproject` file in the given directory.
    ///
    /// # Arguments
    /// * `path` - Path to the **project root directory** to search for the `.pproject` file
    ///
    /// # Panics
    /// Panics if no `.pproject` file is found in the directory
    pub fn get_pproject(path: &PathBuf) -> Result<PhantomProject> {
        let pproject_path = std::fs::read_dir(path)
            .map_err(|e| anyhow::anyhow!("Cannot open project directory '{}': {e}", path.display()))?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "pproject")
            })
            .map(|entry| entry.path())
            .ok_or_else(|| anyhow::anyhow!("No .pproject file found in '{}'", path.display()))?;
        let bytes = std::fs::read(&pproject_path)?;
        let pproject = serde_json::from_slice::<PhantomProject>(&bytes)?;
        Ok(pproject)
    }
}
