use egui::{Color32, Id, Ui};
use glam::{Quat, UVec2, Vec3};
use phantom_core::{
    ecs::{Entity, World},
    reflecton::fields::Field,
};

pub struct FieldContext<'a> {
    pub ui: &'a mut Ui,
    pub world: &'a mut World,
    pub component_name: &'a String,
    pub selected_entity: Entity,
    pub fields: &'a Vec<Field>,
    pub index: usize,
}

impl<'a> FieldContext<'a> {
    pub fn show_f32(&mut self, field_name: &'static str, value: f32) {
        let id = generate_id(self.selected_entity, self.component_name, self.index);

        init_temp(self.ui, id, value);

        let mut value = self.ui.data_mut(|w| w.get_temp::<f32>(id)).unwrap();
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut value).prefix("").speed(0.1))
                .changed()
            {
                ui.data_mut(|w| w.insert_temp::<f32>(id, value));
                let mut new_fields = self.fields.clone();
                new_fields[self.index] =
                    Field::F32(field_name, ui.data(|r| r.get_temp::<f32>(id)).unwrap());
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
        });
    }
    pub fn show_vec3(&mut self, field_name: &'static str, value: Vec3) {
        let mut x = value.x;
        let mut y = value.y;
        let mut z = value.z;

        self.ui.horizontal(|ui| {
            ui.label(field_name);
            // X field
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let new_vec = Vec3 { x, y, z };
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, new_vec);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            // Y field
            if ui
                .add(egui::DragValue::new(&mut y).prefix("Y: ").speed(0.1))
                .changed()
            {
                let new_vec = Vec3 { x, y, z };
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, new_vec);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            // Z field
            if ui
                .add(egui::DragValue::new(&mut z).prefix("Z: ").speed(0.1))
                .changed()
            {
                let new_vec = Vec3 { x, y, z };
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, new_vec);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
        });
    }
    pub fn show_uvec2(&mut self, field_name: &'static str, value: UVec2) {
        let mut x = value.x;
        let mut y = value.y;

        self.ui.horizontal(|ui| {
            ui.label(field_name);
            // X field
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let new_vec = UVec2 { x, y };
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::UVec2(field_name, new_vec);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            // Y field
            if ui
                .add(egui::DragValue::new(&mut y).prefix("Y: ").speed(0.1))
                .changed()
            {
                let new_vec = UVec2 { x, y };
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::UVec2(field_name, new_vec);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
        });
    }
    pub fn show_string(&mut self, field_name: &'static str, value: String) {
        let id = generate_id(self.selected_entity, self.component_name, self.index);
        init_temp(self.ui, id, value);
        let mut text = self.ui.data_mut(|w| w.get_temp::<String>(id)).unwrap();
        let mut response = None;
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            response = Some(ui.text_edit_singleline(&mut text));
        });
        self.ui.data_mut(|w| w.insert_temp(id, text));
        if response.is_some_and(|r| r.lost_focus()) {
            let mut new_fields = self.fields.clone();
            new_fields[self.index] = Field::String(
                field_name,
                self.ui.data_mut(|w| w.get_temp::<String>(id)).unwrap(),
            );
            self.world.set_component_fields(
                self.component_name.clone(),
                self.selected_entity,
                new_fields.clone(),
            );
            self.ui.data_mut(|w| {
                w.remove::<String>(id);
            });
        };
    }
    pub fn show_name_string(&mut self, field_name: &'static str, value: String) {
        let id = generate_id(self.selected_entity, self.component_name, self.index);
        init_temp(self.ui, id, value);
        let mut text = self.ui.data_mut(|w| w.get_temp::<String>(id)).unwrap();

        let response = self.ui.text_edit_singleline(&mut text);
        self.ui.data_mut(|w| w.insert_temp(id, text));
        if response.lost_focus() {
            let mut new_fields = self.fields.clone();
            new_fields[self.index] = Field::NameString(
                field_name,
                self.ui.data_mut(|w| w.get_temp::<String>(id)).unwrap(),
            );
            self.world.set_component_fields(
                self.component_name.clone(),
                self.selected_entity,
                new_fields.clone(),
            );
            self.ui.data_mut(|w| {
                w.remove::<String>(id);
            });
        };
    }
    pub fn show_trans_quat(&mut self, field_name: &'static str, value: Quat) {
        let id = generate_id(self.selected_entity, self.component_name, self.index);

        let euler = value.to_euler(glam::EulerRot::XYZ);
        init_temp(self.ui, id, euler);

        let vec3 = Vec3::new(
            self.ui
                .data(|r| r.get_temp::<(f32, f32, f32)>(id))
                .unwrap()
                .0
                .to_degrees(),
            self.ui
                .data(|r| r.get_temp::<(f32, f32, f32)>(id))
                .unwrap()
                .1
                .to_degrees(),
            self.ui
                .data(|r| r.get_temp::<(f32, f32, f32)>(id))
                .unwrap()
                .2
                .to_degrees(),
        );
        let mut x = vec3.x;
        let mut y = vec3.y;
        let mut z = vec3.z;

        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let new_quat = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    x.to_radians(),
                    y.to_radians(),
                    z.to_radians(),
                );

                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::TransQuat(field_name, new_quat);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
                ui.data_mut(|w| {
                    w.insert_temp::<(f32, f32, f32)>(
                        id,
                        (x.to_radians(), y.to_radians(), z.to_radians()),
                    )
                });
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
                let new_quat = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    x.to_radians(),
                    y.to_radians(),
                    z.to_radians(),
                );

                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::TransQuat(field_name, new_quat);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
                ui.data_mut(|w| {
                    w.insert_temp::<(f32, f32, f32)>(
                        id,
                        (x.to_radians(), y.to_radians(), z.to_radians()),
                    )
                });
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
                let new_quat = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    x.to_radians(),
                    y.to_radians(),
                    z.to_radians(),
                );

                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::TransQuat(field_name, new_quat);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
                ui.data_mut(|w| {
                    w.insert_temp::<(f32, f32, f32)>(
                        id,
                        (x.to_radians(), y.to_radians(), z.to_radians()),
                    )
                });
            };
        });
    }
    pub fn show_color(&mut self, field_name: &'static str, value: [u8; 4]) {
        let id = generate_id(self.selected_entity, self.component_name, self.index);
        let color = Color32::from_rgba_unmultiplied(value[0], value[1], value[2], value[3]);
        init_temp(self.ui, id, color);

        let mut value = self.ui.data(|r| r.get_temp::<Color32>(id)).unwrap();
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui.color_edit_button_srgba(&mut value).changed() {
                ui.data_mut(|w| w.insert_temp::<Color32>(id, value));
                let new_colors = value.to_array();
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Color(field_name, new_colors);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
        });
    }
}

// HELPER FUNCTIONS
fn generate_id(selected_entity: Entity, component_name: &String, index: usize) -> Id {
    Id::new((selected_entity.id, component_name, index))
}

fn init_temp<T: Clone + Send + Sync + 'static>(ui: &mut Ui, id: Id, value: T) {
    if ui.data_mut(|w| w.get_temp::<T>(id)).is_none() {
        ui.data_mut(|w| w.insert_temp::<T>(id, value));
    }
}
