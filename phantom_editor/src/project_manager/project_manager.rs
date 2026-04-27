use std::path::PathBuf;

use anyhow::{Ok, Result};
use phantom_core::ecs::World;
use phantom_project::phantom_project::PhantomProject;

pub struct ProjectManager {}

impl ProjectManager {
    pub fn load(path: PathBuf) -> Result<(PhantomProject, World)> {
        let pproject = get_pproject(&path)?;
        let world_path = path.parent().unwrap().join(&pproject.entry_world);
        let world_bytes = std::fs::read(&world_path)?;
        let world = World::deserialize(&world_bytes);
        Ok((pproject, world))
    }
}

fn get_pproject(path: &PathBuf) -> Result<PhantomProject> {
    let pproject_path = std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "pproject")
        })
        .map(|entry| entry.path())
        .unwrap_or_else(|| {
            log::error!(".pproject cannot be found");
            panic!(".pproject cannot be found");
        });
    let bytes = std::fs::read(&pproject_path)?;
    let pproject = bincode::deserialize::<PhantomProject>(&bytes)?;
    Ok(pproject)
}
