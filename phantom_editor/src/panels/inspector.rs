use std::sync::{Arc, Mutex};

use egui::{Color32, Grid, Id, Label, Layout, RichText, Spacing, TextBuffer, Ui, Vec2};
use glam::{Quat, Vec3};
use log::*;
use phantom_core::{
    ecs::{
        Entity,
        component_registry::{self, COMPONENT_REGISTRY},
    },
    reflecton::fields::{self, Field},
};

use crate::{context::EditorContext, panels::field_wigets::FieldContext, resources::ResourceKey};

pub struct InspectorPanel {}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(ectx) = ui.ctx().data_mut(|w| {
            w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
        }) {
            let mut ectx = ectx.lock().unwrap();
            let selected_entity = ectx.selected_entity;
            if selected_entity.is_some() {
                let world = &mut ectx.active_world;
                let mut components = world.get_component_fields(selected_entity.unwrap());
                components.sort_by(|a, b| match (a.0.as_str(), b.0.as_str()) {
                    ("Transform", _) => std::cmp::Ordering::Less,
                    (_, "Transform") => std::cmp::Ordering::Greater,
                    ("Name", _) => std::cmp::Ordering::Less,
                    (_, "Name") => std::cmp::Ordering::Greater,
                    (x, y) => x.cmp(y),
                });
                // FOR EACH COMPONENT
                for (component_name, fields) in components {
                    ui.label(RichText::new(&component_name).strong());
                    let mut fctx = FieldContext {
                        ui,
                        world,
                        component_name: &component_name,
                        selected_entity: selected_entity.unwrap(),
                        fields: &fields,
                        index: 0,
                    };
                    for (index, field) in fields.iter().enumerate() {
                        fctx.index = index;
                        match field {
                            Field::F32(name, val) => fctx.show_f32(name, *val),
                            Field::Vec3(name, val) => fctx.show_vec3(name, *val),
                            Field::UVec2(name, val) => fctx.show_uvec2(name, *val),
                            Field::NameString(name, val) => {
                                fctx.show_name_string(name, val.clone())
                            }
                            Field::String(name, val) => fctx.show_string(name, val.clone()),
                            Field::TransQuat(name, val) => fctx.show_trans_quat(name, *val),
                            Field::Color(name, val) => fctx.show_color(name, *val),
                            _ => (),
                        }
                    }
                    ui.separator();
                }
                // Add Component Button
                let registered_components = component_registry::get_registered_component_names();
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let response = ui
                        .menu_button("ADD COMPONENT", |ui| {
                            for component in registered_components {
                                if component.as_str() != "Transform" && component.as_str() != "Name"
                                {
                                    if ui.button(component.as_str()).clicked() {
                                        component_registry::add_component_by_name(
                                            world,
                                            selected_entity.unwrap(),
                                            component.as_str(),
                                        );
                                    };
                                }
                            }
                        })
                        .response;
                });
            }
        }
        ui.separator();
    }
}

fn generate_id(selected_entity: Option<Entity>, component_name: &String, index: usize) -> Id {
    Id::new((selected_entity.unwrap().id, component_name, index))
}

fn init_temp<T: Clone + Send + Sync + 'static>(ui: &mut Ui, id: Id, value: T) {
    if ui.data_mut(|w| w.get_temp::<T>(id)).is_none() {
        ui.data_mut(|w| w.insert_temp::<T>(id, value));
    }
}
