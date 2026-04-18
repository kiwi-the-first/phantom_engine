pub use crate::panels::Panels as Tab;
use crate::panels::{
    AssetBrowserPanel, ConsolePanel, HierarchyPanel, InspectorPanel, ViewportPanel,
};
use egui::{Ui, WidgetText};
use egui_dock::TabViewer;

pub struct EditorTabViewer {
    console: ConsolePanel,
    viewport: ViewportPanel,
    hierarchy: HierarchyPanel,
    inspector: InspectorPanel,
    asset_browser: AssetBrowserPanel,
}

impl EditorTabViewer {
    pub fn new() -> Self {
        Self {
            console: ConsolePanel::new(),
            viewport: ViewportPanel::new(),
            hierarchy: HierarchyPanel::new(),
            inspector: InspectorPanel::new(),
            asset_browser: AssetBrowserPanel::new(),
        }
    }
}
impl TabViewer for EditorTabViewer {
    type Tab = Tab;
    // Returns the current `tab`'s title.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Console => self.console.show(ui),
            Tab::Viewport => self.viewport.show(ui),
            Tab::Hierarchy => self.hierarchy.show(ui),
            Tab::Inspector => self.inspector.show(ui),
            Tab::AssetBrowser => self.asset_browser.show(ui),
        };
    }
}
