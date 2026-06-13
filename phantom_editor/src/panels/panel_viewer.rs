use crate::context::panel_context::PanelContext;
pub use crate::panels::Panels as Tab;
use crate::panels::{
    AssetBrowserPanel, ConsolePanel, HierarchyPanel, InspectorPanel, ViewportState, ViewportPanel,
};
use crate::{actions::Actions, context::EditorContext};
use egui::{Ui, WidgetText};
use egui_dock::TabViewer;

/// Transient per-frame viewer. Holds borrows of the editor's shared state for the
/// duration of a single dock draw — never stored anywhere long-lived.
pub struct PanelViewer<'a> {
    pub editor_ctx: &'a mut EditorContext,
    pub actions: &'a mut Actions,
    pub panel_context: &'a mut PanelContext,
}

impl<'a> TabViewer for PanelViewer<'a> {
    type Tab = Tab;
    // Returns the current `tab`'s title.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Console => ConsolePanel::show(ui, self.editor_ctx),
            Tab::Viewport => ViewportPanel::show(ui, self.editor_ctx, self.panel_context),
            Tab::Hierarchy => HierarchyPanel::show(ui, self.editor_ctx, self.actions),
            Tab::Inspector => InspectorPanel::show(ui, self.editor_ctx),
            Tab::AssetBrowser => AssetBrowserPanel::show(ui, self.editor_ctx, self.panel_context),
        };
    }
}
