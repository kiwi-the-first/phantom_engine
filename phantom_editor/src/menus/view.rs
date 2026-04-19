use std::collections::HashMap;

use egui::Ui;

use crate::workspaces::{WorkspaceConfig, workspace};

pub struct ViewMenu {}

impl ViewMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        ui: &mut Ui,
        avalible_workspaces: &HashMap<String, WorkspaceConfig>,
    ) -> Option<String> {
        let mut workspace_to_open = None;

        ui.menu_button("Workspaces", |ui| {
            for (name, _) in avalible_workspaces {
                if ui.button(name).clicked() {
                    workspace_to_open = Some(name.clone())
                }
            }
        });

        workspace_to_open
    }
}
