use egui::Ui;
use phantom_build::BuildSystem;
use phantom_project::project_manager::project_manager::ProjectManager;

use crate::context::EditorContext;

pub struct FileMenu {}

impl FileMenu {
    pub fn show(ui: &mut Ui, editor: &mut EditorContext) {
        if ui.button("Save Project").clicked() {
            if let Err(e) = ProjectManager::save(editor.project_path.clone(), &editor.active_world) {
                log::error!("FAILED TO SAVE PROJECT! {e}");
            }
        };
        if ui.button("Build Game").clicked() {
            if let Err(e) = BuildSystem::build(editor.project_path.clone()) {
                log::error!("FAILED TO BUILD PROJECT {e}");
            }
        }
        if ui.button("Reload Scripts").clicked() {
            if let Err(e) = editor.reload_project() {
                log::error!("FAILED TO RELOAD SCRIPTS {e}");
            }
        }
    }
}
