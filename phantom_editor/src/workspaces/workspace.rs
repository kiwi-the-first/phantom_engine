use egui::Panel;
use egui_dock::DockState;

use crate::{panels::PanelViewer, panels::Panels};

pub struct Workspace {
    pub name: String,
    pub panel_dock_state: DockState<Panels>,
    pub panel_viewer: PanelViewer,
}

impl Workspace {
    pub fn new(name: String, panels: Vec<Panels>) -> Self {
        let panel_dock_state = DockState::new(panels);
        let panel_viewer = PanelViewer::new();

        Self {
            name,
            panel_dock_state,
            panel_viewer,
        }
    }
}
