use std::collections::HashMap;

use egui::Ui;

use crate::workspaces::{BuiltInWorkspace, WorkspaceConfig, workspace};

pub enum ViewMenuAction {
    OpenWorkspace(String),
    SaveLayout,
    LoadDefaultLayout(BuiltInWorkspace),
    LoadCustomLayout(String),
}

pub struct ViewMenu {}

impl ViewMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        ui: &mut Ui,
        avalible_workspaces: &HashMap<String, WorkspaceConfig>,
        active_workspace_name: String,
        active_workspace_type: Option<BuiltInWorkspace>,
    ) -> Option<ViewMenuAction> {
        let mut view_action = None;
        ui.menu_button("Workspaces", |ui| {
            for (name, _) in avalible_workspaces {
                if ui.button(name).clicked() {
                    view_action = Some(ViewMenuAction::OpenWorkspace(name.clone()));
                }
            }
        });
        if ui
            .button(format!("Save Custom {} layout", active_workspace_name))
            .clicked()
        {
            view_action = Some(ViewMenuAction::SaveLayout);
        };
        if ui
            .button(format!("Load default {} layout", active_workspace_name))
            .clicked()
        {
            view_action = Some(ViewMenuAction::LoadDefaultLayout(
                active_workspace_type.unwrap(),
            ))
        }
        if ui
            .button(format!("Load Custom {} layout", active_workspace_name))
            .clicked()
        {
            view_action = Some(ViewMenuAction::LoadCustomLayout(
                active_workspace_name.clone(),
            ))
        }

        view_action
    }
}
