use std::collections::HashMap;

use egui::Ui;

use crate::workspaces::{WorkspaceConfig, workspace};

pub enum ViewMenuAction {
    OpenWorkspace(String),
    SaveLayout,
}

pub struct ViewMenu {}

impl ViewMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        ui: &mut Ui,
        avalible_workspaces: &HashMap<String, WorkspaceConfig>,
    ) -> Option<ViewMenuAction> {
        let mut view_action = None;
        ui.menu_button("Workspaces", |ui| {
            for (name, _) in avalible_workspaces {
                if ui.button(name).clicked() {
                    view_action = Some(ViewMenuAction::OpenWorkspace(name.clone()));
                }
            }
        });
        if ui.button("Save Active Workspace Layout").clicked() {
            view_action = Some(ViewMenuAction::SaveLayout);
        };

        view_action
    }
}
