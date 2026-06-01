use std::path::{Path, PathBuf};

use anyhow::Result;
use phantom_project::project_manager::project_manager::ProjectManager;

use crate::{app::editor_app::EditorApp, context::EditorContext};

pub mod actions;
pub mod app;
pub mod context;
pub mod egui;
pub mod logger;
pub mod menus;
pub mod panels;
pub mod persitance;
pub mod resources;
pub mod workspaces;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = PathBuf::from(args.get(1).unwrap());
    EditorApp::run(path).unwrap();
    Ok(())
}
