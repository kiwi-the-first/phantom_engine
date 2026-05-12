use std::sync::{Arc, Mutex};

use egui::{Grid, Id, Label, RichText, Spacing, Ui, Vec2};
use glam::{Quat, Vec3};
use log::*;
use phantom_core::reflecton::fields::{self, Field};
use winit::platform::x11;

use crate::{context::EditorContext, resources::ResourceKey};

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
                components.sort_by_key(|(name, _)| match name.as_str() {
                    /// Make sure Name, Transform are always at the top of the component list
                    "Name" => 0,
                    "Transform" => 1,
                    _ => 2,
                });
                // FOR EACH COMPONENT
                for (component_name, fields) in components {
                    ui.label(RichText::new(&component_name).strong());
                    for (index, field) in fields.iter().enumerate() {
                        match field {
                            /// This field is for the built in Name Component
                            /// it removes the field_name from display
                            Field::NameString(field_name, string) => {
                                let id = Id::new((selected_entity.unwrap().id, index));
                                if ui.data_mut(|w| w.get_temp::<String>(id)).is_none() {
                                    //insert feild name
                                    ui.data_mut(|w| w.insert_temp::<&'static str>(id, field_name));
                                    //insert data
                                    ui.data_mut(|w| w.insert_temp::<String>(id, string.clone()));
                                }
                                let mut text = ui.data_mut(|w| w.get_temp::<String>(id)).unwrap();

                                let response = ui.text_edit_singleline(&mut text);
                                ui.data_mut(|w| w.insert_temp(id, text));
                                if response.lost_focus() {
                                    let mut new_fields = fields.clone();
                                    new_fields[index] = Field::String(
                                        ui.data_mut(|w| w.get_temp::<&'static str>(id).unwrap()),
                                        ui.data_mut(|w| w.get_temp::<String>(id)).unwrap(),
                                    );
                                    world.set_component_fields(
                                        component_name.clone(),
                                        selected_entity.unwrap(),
                                        new_fields.clone(),
                                    );
                                    ui.data_mut(|w| {
                                        w.remove::<String>(id);
                                        w.remove::<&'static str>(id);
                                    });
                                };
                            }
                            Field::String(field_name, string) => {
                                let id = Id::new((selected_entity.unwrap().id, index));
                                if ui.data_mut(|w| w.get_temp::<String>(id)).is_none() {
                                    //insert feild name
                                    ui.data_mut(|w| w.insert_temp::<&'static str>(id, field_name));
                                    //insert data
                                    ui.data_mut(|w| w.insert_temp::<String>(id, string.clone()));
                                }
                                let mut text = ui.data_mut(|w| w.get_temp::<String>(id)).unwrap();
                                let mut response = None;
                                ui.horizontal(|ui| {
                                    ui.label(*field_name);
                                    response = Some(ui.text_edit_singleline(&mut text));
                                });
                                ui.data_mut(|w| w.insert_temp(id, text));
                                if response.is_some_and(|r| r.lost_focus()) {
                                    let mut new_fields = fields.clone();
                                    new_fields[index] = Field::String(
                                        ui.data_mut(|w| w.get_temp::<&'static str>(id).unwrap()),
                                        ui.data_mut(|w| w.get_temp::<String>(id)).unwrap(),
                                    );
                                    trace!(
                                        "set {} fields",
                                        ui.data_mut(|w| w.get_temp::<String>(id)).unwrap()
                                    );
                                    world.set_component_fields(
                                        component_name.clone(),
                                        selected_entity.unwrap(),
                                        new_fields.clone(),
                                    );
                                    ui.data_mut(|w| {
                                        w.remove::<String>(id);
                                        w.remove::<&'static str>(id);
                                    });
                                };
                            }
                            Field::Vec3(field_name, vec3) => {
                                
                                let mut x = vec3.x;
                                let mut y = vec3.y;
                                let mut z = vec3.z;

                                ui.horizontal(|ui| {
                                    ui.label(*field_name);
                                    if ui
                                        .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                                        .changed()
                                    {
                                        let new_vec = Vec3 { x, y, z };
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::Vec3(field_name, new_vec);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                    if ui
                                        .add(egui::DragValue::new(&mut y).prefix("Y: ").speed(0.1))
                                        .changed()
                                    {
                                        let new_vec = Vec3 { x, y, z };
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::Vec3(field_name, new_vec);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                    if ui
                                        .add(egui::DragValue::new(&mut z).prefix("Z: ").speed(0.1))
                                        .changed()
                                    {
                                        let new_vec = Vec3 { x, y, z };
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::Vec3(field_name, new_vec);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                });
                            }
                            /// This field is for the built in Transform Component
                            /// it converts the Quat into Euler for modification 
                            Field::TransQuat(field_name, quat) => {
                                let euler = quat.to_euler(glam::EulerRot::XYZ);

                                let vec3 = Vec3::new(
                                    euler.0.to_degrees(),
                                    euler.1.to_degrees(),
                                    euler.2.to_degrees(),
                                );
                                let mut x = vec3.x;
                                let mut y = vec3.y;
                                let mut z = vec3.z;

                                ui.horizontal(|ui| {
                                    ui.label(*field_name);
                                    if ui
                                        .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                                        .changed()
                                    {
                                        let new_quat =
                                            Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::TransQuat(field_name, new_quat);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut y)
                                                .prefix("Y: ")
                                                .speed(0.1)
                                                .range(-360..=360),
                                        )
                                        .changed()
                                    {
                                        let new_quat =
                                            Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::TransQuat(field_name, new_quat);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut z)
                                                .prefix("Z: ")
                                                .speed(0.1)
                                                .range(-360..=360),
                                        )
                                        .changed()
                                    {
                                        let new_quat =
                                            Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
                                        let mut new_fields = fields.clone();
                                        new_fields[index] = Field::TransQuat(field_name, new_quat);
                                        world.set_component_fields(
                                            component_name.clone(),
                                            selected_entity.unwrap(),
                                            new_fields,
                                        );
                                    };
                                });
                            }
                            Field::Quat(field_name, quat) => {
                                ui.label(format!("{}", quat));
                            }
                            Field::F32(field_name, f32) => {
                                ui.label(format!("{}", f32));
                            }
                            _ => {}
                        };
                    }
                    ui.end_row();
                }
            }
        }
        ui.separator();
    }
}
