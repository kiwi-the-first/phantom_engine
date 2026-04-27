use anyhow::Result;
use phantom_core::ecs::World;
use std::{fs::create_dir_all, io::Write, path::PathBuf, process::Command};

use crate::phantom_project::PhantomProject;

pub fn create_project(name: String, path: PathBuf) -> anyhow::Result<()> {
    // Create Crate
    Command::new("cargo")
        .args(["new", "--lib", path.to_str().unwrap()])
        .status()?;

    let world = World::new();
    let world_data = world.serialize();

    let world_path = path.join("worlds");
    let world_name = world_path.join("initial_world.pworld");
    std::fs::create_dir_all(&world_path)?;
    let mut file = std::fs::File::create(&world_name)?;
    file.write_all(&world_data)?;

    create_pproject_file(name, path, world_name)?;

    Ok(())
}

fn create_pproject_file(
    name: String,
    path: PathBuf,
    world_path: PathBuf,
) -> Result<PhantomProject> {
    let version = phantom_core::constants::VERSION;
    let pproject = PhantomProject {
        name: name,
        version: version.to_string(),
        entry_world: world_path,
    };
    let bytes = bincode::serialize::<PhantomProject>(&pproject).unwrap();
    std::fs::write(path.join(format!("{}.pproject", &pproject.name)), bytes)?;
    Ok(pproject)
}
