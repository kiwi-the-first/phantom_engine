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

    let path = std::env::var("PHANTOM_PROJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::args()
                .nth(1)
                .map(PathBuf::from)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Missing project path. Pass the project root as the first argument or set PHANTOM_PROJECT_ROOT."
                    )
                })
        })?;
    EditorApp::run(path)?;
    Ok(())
}
