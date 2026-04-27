use std::sync::{Arc, Mutex};

use egui::{Id, Sense, Ui};

use crate::actions::{Actions, Command, commands::summon_entity::CommandSummonEntity};

pub struct HierarchyPanel {}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let rect = ui.content_rect();
        let response = ui.allocate_rect(rect, Sense::click());
        response.context_menu(|ui| {
            if ui.button("Summon Empty").clicked() {
                if let Some(actions) = ui
                    .ctx()
                    .data_mut(|w| w.get_temp::<Arc<Mutex<Actions>>>(Id::new("Actions")))
                {
                    let mut actions = actions.lock().unwrap();
                    actions.do_command(Box::new(CommandSummonEntity {}));
                }
            }
        });
        ui.separator();
    }
}
