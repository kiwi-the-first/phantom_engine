use egui::{Ui, WidgetText};
use egui_dock::TabViewer;

pub type Tab = String;

pub struct EditorTabViewer {}

impl TabViewer for EditorTabViewer {
    type Tab = Tab;
    // Returns the current `tab`'s title.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }
}
