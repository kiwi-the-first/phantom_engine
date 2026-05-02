use std::sync::{Arc, Mutex};

use egui::{Color32, Id, Label, ScrollArea, Sense, Ui};
use log::trace;
use phantom_core::ecs::{Entity, components::Name};

use crate::{
    actions::{Actions, Command, commands::summon_entity::CommandSummonEntity},
    context::EditorContext,
    resources::ResourceKey,
};

pub struct HierarchyPanel {}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            let ctx = ui
                .ctx()
                .data(|r| {
                    r.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
                })
                .unwrap();

            let entities_and_names: Vec<(Entity, String)> = {
                let editor_ctx = ctx.lock().unwrap();
                let world = &editor_ctx.active_world;
                world
                    .query_with::<Name>()
                    .iter()
                    .map(|&entity| {
                        let name = world.get_component::<Name>(entity).unwrap();
                        (entity, name.name.clone())
                    })
                    .collect()
            };

            for (entity, name) in entities_and_names {
                let selected = ctx.lock().unwrap().selected_entity == Some(entity);
                // Make font black if selected
                let font_color = if selected {
                    Color32::BLACK
                } else {
                    ui.visuals().text_color()
                };

                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), Sense::click());

                if selected {
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
                } else if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().widgets.hovered.bg_fill);
                }

                ui.painter().text(
                    rect.left_center() + egui::vec2(4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &name,
                    egui::FontId::default(),
                    font_color,
                );

                if response.clicked() {
                    trace!("Selected entity: {}", entity);
                    ctx.lock().unwrap().selected_entity = Some(entity);
                }

                response.context_menu(|ui| {
                    standard_context_menu(ui);
                });
            }
            // Outside of list
            let size = ui.available_size();
            let space = ui.allocate_space(size);
            ui.interact(space.1, space.0, Sense::click())
                .context_menu(|ui| {
                    standard_context_menu(ui);
                });
        });
    }
}

fn standard_context_menu(ui: &mut Ui) {
    if ui.button("Summon Empty").clicked() {
        if let Some(actions) = ui
            .ctx()
            .data_mut(|w| w.get_temp::<Arc<Mutex<Actions>>>(Id::new(ResourceKey::Actions)))
        {
            let mut actions = actions.lock().unwrap();
            let ctx = ui
                .ctx()
                .data(|r| {
                    r.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
                })
                .unwrap();

            actions.do_command(Box::new(CommandSummonEntity::new()), &ctx);
        }
    }
}
