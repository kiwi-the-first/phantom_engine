use egui::Ui;
use phantom_build::BuildSystem;
use phantom_project::project_manager::project_manager::ProjectManager;

use crate::context::EditorContext;

pub struct FileMenu {}

impl FileMenu {
    pub fn show(ui: &mut Ui, ectx: &mut EditorContext) {
        if ui.button("Save Project").clicked() {
            if ectx.is_playing {
                log::warn!("YOU MUST EXIT PLAY MODE BEFORE SAVING!");
                return;
            };
            if let Err(e) = ProjectManager::save(ectx.project_path.clone(), &ectx.active_world) {
                log::error!("FAILED TO SAVE PROJECT! {e}");
            }
        };
        if ui.button("Build Game").clicked() {
            if ectx.is_playing {
                log::warn!("YOU MUST EXIT PLAY MODE BEFORE BUILDING!");
                return;
            };
            ectx.build_project()
        }
        if ui.button("Reload Scripts").clicked() {
            if ectx.is_playing {
                log::warn!("YOU MUST EXIT PLAY MODE BEFORE RELOADING!");
                return;
            };
            if let Err(e) = ectx.reload_project() {
                log::error!("FAILED TO RELOAD SCRIPTS {e}");
            }
        }
    }
}
