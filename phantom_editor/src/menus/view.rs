use egui::Ui;

use crate::dock::DockManager;

pub struct ViewMenu {}

impl ViewMenu {
    /// Draws the View menu. Workspace/layout operations are delegated to the
    /// `DockManager`, which owns the dock state.
    pub fn show(ui: &mut Ui, dock: &mut DockManager) {
        let active = dock.active_workspace_name().unwrap_or_default();

        ui.menu_button("Workspaces", |ui| {
            for name in dock.available_names() {
                if ui.button(name).clicked() {
                    dock.open(name);
                }
            }
        });

        if ui
            .button(format!("Save Custom {active} layout"))
            .clicked()
        {
            dock.save_active_layout();
        }
        if ui
            .button(format!("Load default {active} layout"))
            .clicked()
        {
            dock.load_active_default_layout();
        }
        if ui
            .button(format!("Load Custom {active} layout"))
            .clicked()
        {
            dock.load_active_custom_layout();
        }
    }
}
