use crate::{panels::Panels, workspaces::WorkspaceKind};

#[derive(Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub kind: WorkspaceKind,
    pub panels: Vec<Panels>,
}
