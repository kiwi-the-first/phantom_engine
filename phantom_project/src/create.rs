use anyhow::Result;
use phantom_core::ecs::{
    World,
    components::{Name, camera::Camera},
};
use std::{fs::create_dir_all, io::Write, path::PathBuf, process::Command};

use crate::phantom_project::PhantomProject;

pub fn create_project(name: String, path: PathBuf) -> anyhow::Result<()> {
    // Create Crate
    Command::new("cargo")
        .args(["new", "--lib", path.to_str().unwrap()])
        .status()?;

    //Create World File and Folder
    let mut world = World::new();

    // Setup Main Camera
    let camera = world.summon();
    let name_comp = world.get_component_mut::<Name>(camera).unwrap();
    name_comp.name = "Main Camera".to_string();
    let camera_comp = world.add_component::<Camera>(camera, Camera::default());
    camera_comp.zoom = 100.0;

    //Save World File
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
    let path = path.canonicalize()?;
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
