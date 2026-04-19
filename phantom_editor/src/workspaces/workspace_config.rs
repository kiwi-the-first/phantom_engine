use crate::{panels::Panels, workspaces::WorkspaceKind};

pub struct WorkspaceConfig {
    pub name: String,
    pub kind: WorkspaceKind,
    pub panels: Vec<Panels>,
}
