use egui::{Color32, ScrollArea, Sense, Ui};
use log::trace;
use phantom_core::ecs::{Entity, components::Name};

use crate::{
    actions::{Actions, commands::summon_entity::CommandSummonEntity},
    context::EditorContext,
};

pub struct HierarchyPanel {}

impl HierarchyPanel {
    pub fn show(ui: &mut Ui, ectx: &mut EditorContext, actions: &mut Actions) {
        ScrollArea::vertical().show(ui, |ui| {
            let entities_and_names: Vec<(Entity, String)> = {
                let world = &ectx.active_world;
                world
                    .query_with::<Name>()
                    .iter()
                    .map(|&entity| {
                        let name = world.get_component::<Name>(entity).unwrap();
                        (entity, name.name.clone())
                    })
                    .collect()
            };
            let selected_entity = ectx.selected_entity;

            let mut clicked: Option<Entity> = None;
            let mut to_delete: Option<Entity> = None;

            for (entity, name) in entities_and_names {
                let selected = selected_entity == Some(entity);
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
                    clicked = Some(entity);
                }

                response.context_menu(|ui| {
                    standard_context_menu(ui, actions, ectx);
                    if ui.button("Delete entity").clicked() {
                        to_delete = Some(entity);
                    }
                });
            }
            // Outside of list
            let size = ui.available_size();
            let space = ui.allocate_space(size);
            ui.interact(space.1, space.0, Sense::click())
                .context_menu(|ui| {
                    standard_context_menu(ui, actions, ectx);
                });

            if let Some(entity) = clicked {
                ectx.selected_entity = Some(entity);
            }
            if let Some(entity) = to_delete {
                trace!("Deleted entity: {}", entity);
                ectx.active_world.destroy(entity);
            }
        });
    }
}

/// Shared right-click menu. Summons a new empty entity through the action stack.
fn standard_context_menu(ui: &mut Ui, actions: &mut Actions, ectx: &mut EditorContext) {
    if ui.button("Summon Empty").clicked() {
        actions.do_command(Box::new(CommandSummonEntity::new()), ectx);
    }
}
