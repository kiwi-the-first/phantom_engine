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

    // Write Cargo.toml with phantom_core dependency and cdylib
    let engine_path = match std::env::var("PHANTOM_ENGINE_PATH") {
        Ok(p) => p,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&path);
            anyhow::bail!("PHANTOM_ENGINE_PATH environment variable not set");
        }
    };
    let cargo_toml = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nphantom_core = {{ path = \"{}/phantom_core\" }}\n\n[lib]\ncrate-type = [\"cdylib\"]\n",
        name, engine_path
    );
    std::fs::write(path.join("Cargo.toml"), cargo_toml)?;

    // Write default lib.rs
    std::fs::write(
        path.join("src").join("lib.rs"),
        "use phantom_core::phantom_macros::phantom_register;\n\nphantom_register!();\n",
    )?;

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
