use crate::panels::{PanelViewer, Viewport};
use crate::workspaces::Workspace;
use crate::{actions::Actions, context::EditorContext};

use egui::{Id, Ui, WidgetText};
use egui_dock::{DockArea, TabViewer};

/// Transient per-frame viewer for the outer (workspace) dock. Threads the editor's
/// shared-state borrows down into a freshly-built `PanelViewer` for the inner dock.
pub struct WorkspaceViewer<'a> {
    pub editor: &'a mut EditorContext,
    pub actions: &'a mut Actions,
    pub viewport: &'a mut Viewport,
}

impl<'a> TabViewer for WorkspaceViewer<'a> {
    type Tab = Workspace;
    // Returns the current `tab`'s title.
    fn title(&mut self, workspace: &mut Self::Tab) -> WidgetText {
        workspace.name.as_str().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, workspace: &mut Self::Tab) {
        let mut panels = PanelViewer {
            editor_ctx: &mut *self.editor,
            actions: &mut *self.actions,
            viewport: &mut *self.viewport,
        };
        DockArea::new(&mut workspace.panel_dock_state)
            .id(Id::new(&workspace.name))
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .show_inside(ui, &mut panels);
    }
}
