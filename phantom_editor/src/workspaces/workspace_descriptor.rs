use crate::{panels::Panels, workspaces::BuiltInWorkspace};

#[derive(Clone)]
pub struct WorkspaceDescriptor {
    pub name: &'static str,
    pub kind: BuiltInWorkspace,
    pub panels: Vec<Panels>,
}
