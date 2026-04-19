use crate::workspaces::Workspace;

use egui::{Id, Ui, WidgetText};
use egui_dock::{DockArea, TabViewer};

pub struct WorkspaceViewer {}

impl WorkspaceViewer {
    pub fn new() -> Self {
        Self {}
    }
}
impl TabViewer for WorkspaceViewer {
    type Tab = Workspace;
    // Returns the current `tab`'s title.
    fn title(&mut self, workspace: &mut Self::Tab) -> WidgetText {
        workspace.name.as_str().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, workspace: &mut Self::Tab) {
        DockArea::new(&mut workspace.panel_dock_state)
            .id(Id::new(&workspace.name))
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .show_inside(ui, &mut workspace.panel_viewer);
    }
}
