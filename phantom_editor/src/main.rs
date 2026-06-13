use std::path::PathBuf;

use anyhow::Result;

use crate::app::editor_app::EditorApp;

pub mod actions;
pub mod app;
pub mod context;
pub mod dock;
pub mod egui;
pub mod logger;
pub mod menus;
pub mod panels;
pub mod persitance;
pub mod shortcuts;
pub mod theme;
pub mod top_bar;
pub mod workspaces;

fn main() -> Result<()> {
    // Enables drag and drop on linux
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let args: Vec<String> = std::env::args().collect();
    let path = PathBuf::from(args.get(1).unwrap());
    EditorApp::run(path).unwrap();
    Ok(())
}
