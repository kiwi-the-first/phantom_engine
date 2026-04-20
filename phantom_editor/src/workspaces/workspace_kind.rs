use crate::workspaces::BuiltInWorkspace;

#[derive(Clone, Copy)]
pub enum WorkspaceKind {
    BuiltIn(BuiltInWorkspace),
    Custom,
}
