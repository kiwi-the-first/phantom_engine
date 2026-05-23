use std::path::{Path, PathBuf};

use anyhow::{Ok, Result};
use egui_dock::DockState;
use phantom_common::dirs;

use crate::{panels::Panels, workspaces::BuiltInWorkspace};

pub fn save(name: String, dock_state: &DockState<Panels>) -> Result<()> {
    let json = serde_json::to_string_pretty(dock_state)?;
    let dir =
        dirs::SystemDirs::config().ok_or(anyhow::anyhow!("failed to load find config path"))?;

    let layout_dir = dir.join("layouts");
    std::fs::create_dir_all(&layout_dir)?;
    std::fs::write(layout_dir.join(format!("{}.json", name)), json)?;
    Ok(())
}

pub fn load(name: String) -> Result<DockState<Panels>> {
    let dir =
        dirs::SystemDirs::config().ok_or(anyhow::anyhow!("failed to load find config path"))?;
    let layout_dir = dir.join(format!("layouts/{}.json", name));
    let file = std::fs::read_to_string(layout_dir)?;

    Ok(serde_json::from_str(file.as_str())?)
}

pub fn load_default(workspace_type: BuiltInWorkspace) -> Result<DockState<Panels>> {
    match workspace_type {
        BuiltInWorkspace::LevelEditor => {
            let str = include_str!("../defaults/default_level_editor.json");
            Ok(serde_json::from_str(str)?)
        }
    }
}
