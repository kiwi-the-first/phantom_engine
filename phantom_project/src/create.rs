use anyhow;
use std::{fs::create_dir_all, path::PathBuf, process::Command};

pub fn create_project(name: String, path: PathBuf) -> anyhow::Result<()> {
    Command::new("cargo")
        .args(["new", "--lib", path.to_str().unwrap()])
        .status()?;
    Ok(())
}
