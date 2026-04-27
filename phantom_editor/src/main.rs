use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    app::editor_app::EditorApp, context::EditorContext,
    project_manager::project_manager::ProjectManager,
};

pub mod actions;
pub mod app;
pub mod context;
pub mod egui;
pub mod menus;
pub mod panels;
pub mod persitance;
pub mod project_manager;
pub mod render_resoruces;
pub mod workspaces;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    println!("path: {}", args.get(1).unwrap());
    let path = PathBuf::from(args.get(1).unwrap());
    let (project, init_world) = ProjectManager::load(path)?;
    let editor_context = EditorContext {
        project: project,
        active_world: init_world,
    };

    EditorApp::run(editor_context).unwrap();
    Ok(())
}
