use egui::{Layout, RichText, Ui};
use phantom_core::ecs::component_registry;
use phantom_core::reflecton::fields::Field;

use crate::{context::EditorContext, panels::field_wigets::FieldContext};

pub struct InspectorPanel {}

impl InspectorPanel {
    pub fn show(ui: &mut Ui, ectx: &mut EditorContext) {
        {
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

                let mut component_to_remove: Option<String> = None;

                for (component_name, fields) in components {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&component_name).strong());
                        if component_name != "Transform" && component_name != "Name" {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("X").clicked() {
                                    component_to_remove = Some(component_name.clone());
                                }
                            });
                        }
                    });

                    let mut fctx = FieldContext {
                        ui,
                        world,
                        component_name: &component_name,
                        selected_entity: selected_entity.unwrap(),
                        fields: &fields,
                        index: 0,
                    };

                    if component_name == "Animator" {
                        fctx.show_animator(&ectx.asset_manager);
                    } else {
                        if fields.is_empty() {
                            fctx.ui.label("[ No exposed fields ]");
                        }
                        for (index, field) in fields.iter().enumerate() {
                            fctx.index = index;
                            match field {
                                Field::Bool(name, val) => fctx.show_bool(name, *val),
                                Field::F32(name, val) => fctx.show_f32(name, *val),
                                Field::I32(name, val) => fctx.show_i32(name, *val),
                                Field::U32(name, val) => fctx.show_u32(name, *val),
                                Field::Vec2(name, val) => fctx.show_vec2(name, *val),
                                Field::Vec3(name, val) => fctx.show_vec3(name, *val),
                                Field::UVec2(name, val) => fctx.show_uvec2(name, *val),
                                Field::NameString(name, val) => fctx.show_name_string(name, val.clone()),
                                Field::String(name, val) => fctx.show_string(name, val.clone()),
                                Field::TransQuat(name, val) => fctx.show_trans_quat(name, *val),
                                Field::Color(name, val) => fctx.show_color(name, *val),
                                Field::Sprite(name, val) => fctx.show_sprite(&ectx.asset_manager, name, *val),
                                _ => (),
                            }
                        }
                    }
                    ui.separator();
                }

                if let Some(component_name) = component_to_remove {
                    world.remove_component_by_name(selected_entity.unwrap(), &component_name);
                }

                let registered_components = component_registry::get_registered_component_names();
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.menu_button("ADD COMPONENT", |ui| {
                        for component in registered_components {
                            if component.as_str() != "Transform" && component.as_str() != "Name" {
                                if ui.button(component.as_str()).clicked() {
                                    component_registry::add_component_by_name(
                                        world,
                                        selected_entity.unwrap(),
                                        component.as_str(),
                                    );
                                };
                            }
                        }
                    });
                });
            }
        }
        ui.separator();
    }
}
