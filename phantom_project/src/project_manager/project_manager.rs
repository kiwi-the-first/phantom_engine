use std::io::Write;
use std::path::PathBuf;

use crate::phantom_project::PhantomProject;
use anyhow::{Ok, Result};
use phantom_core::ecs::World;

pub struct ProjectManager {}

impl ProjectManager {
    pub fn load(path: PathBuf) -> Result<(PhantomProject, World)> {
        let pproject = get_pproject(&path)?;
        let world_path = path.parent().unwrap().join(&pproject.entry_world);
        let world_bytes = std::fs::read(&world_path)?;
        let world = World::deserialize(&world_bytes);
        Ok((pproject, world))
    }

    pub fn save(path: PathBuf, world: &World) -> anyhow::Result<()> {
        let pproject = get_pproject(&path)?;
        let world_path = path.parent().unwrap().join(&pproject.entry_world);
        let world_data = world.serialize();
        let mut file = std::fs::File::create(&world_path)?;
        file.write_all(&world_data)?;
        Ok(())
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
