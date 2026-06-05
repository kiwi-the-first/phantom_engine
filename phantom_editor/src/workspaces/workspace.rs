use egui_dock::DockState;

use crate::panels::Panels;

pub struct Workspace {
    pub name: String,
    pub panel_dock_state: DockState<Panels>,
}

impl Workspace {
    pub fn new(name: String, panels: Vec<Panels>) -> Self {
        let panel_dock_state = DockState::new(panels);

        Self {
            name,
            panel_dock_state,
        }
    }
}
