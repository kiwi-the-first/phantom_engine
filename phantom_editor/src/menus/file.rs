use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use egui::{Id, Ui};
use phantom_build::BuildSystem;
use phantom_project::project_manager::project_manager::ProjectManager;

use crate::{context::EditorContext, resources::ResourceKey};

pub struct FileMenu {}

impl FileMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(ui: &mut Ui) {
        if let Some(ectx) = ui.ctx().data_mut(|w| {
            w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
        }) {
            let ectx_lock = ectx.lock().unwrap();
            if ui.button("Save Project").clicked() {
                let path = &ectx_lock.project_path;
                let world = &ectx_lock.active_world;
                if let Err(e) = ProjectManager::save(path.clone(), &world) {
                    log::error!("FAILED TO SAVE PROJECT! {e}");
                }
            };
            if ui.button("Build Game").clicked() {
                let path = &ectx_lock.project_path;
                if let Err(e) = BuildSystem::build(path.clone()) {
                    log::error!("FAILED TO BUILD PROJECT {e}");
                }
            }
        };
    }
}
