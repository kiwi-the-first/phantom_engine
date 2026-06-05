use crate::{
    panels::Panels,
    workspaces::{BuiltInWorkspace, WorkspaceDescriptor},
};

/// Factory for the engine's built-in workspace layouts.
pub struct WorkspacePresets;

impl WorkspacePresets {
    pub fn level_editor() -> WorkspaceDescriptor {
        WorkspaceDescriptor {
            name: "Level Editor",
            kind: BuiltInWorkspace::LevelEditor,
            panels: vec![
                Panels::Viewport,
                Panels::Hierarchy,
                Panels::Inspector,
                Panels::Console,
                Panels::AssetBrowser,
            ],
        }
    }

    /// Every built-in preset, registered into the `DockManager` at startup.
    pub fn all() -> Vec<WorkspaceDescriptor> {
        vec![Self::level_editor()]
    }
}
