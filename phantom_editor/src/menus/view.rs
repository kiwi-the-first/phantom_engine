use egui::{Id, Ui};

use crate::{resources::ResourceKey, workspaces::BuiltInWorkspace};

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

    pub fn show(ui: &mut Ui) -> Option<ViewMenuAction> {
        let mut view_action = None;
        let available_workspaces = ui
            .ctx()
            .data(|r| r.get_temp::<Vec<String>>(Id::new(ResourceKey::AvailableWorkspaces)))
            .unwrap();

        let active_workspace_name = ui
            .ctx()
            .data(|r| r.get_temp::<String>(Id::new(ResourceKey::ActiveWorkspaceName)))
            .unwrap();

        let active_workspace_type = ui
            .ctx()
            .data(|r| {
                r.get_temp::<Option<BuiltInWorkspace>>(Id::new(
                    ResourceKey::ActiveWorkspaceBuiltInType,
                ))
            })
            .unwrap();

        ui.menu_button("Workspaces", |ui| {
            for name in available_workspaces {
                if ui.button(&name).clicked() {
                    view_action = Some(ViewMenuAction::OpenWorkspace(name));
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
