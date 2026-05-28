use egui::{Color32, Ui};
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
    pub fn show_f32(&mut self, field_name: &'static str, mut value: f32) {
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut value).speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::F32(field_name, value);
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
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, Vec3 { x, y, z });
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            if ui
                .add(egui::DragValue::new(&mut y).prefix("Y: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, Vec3 { x, y, z });
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            if ui
                .add(egui::DragValue::new(&mut z).prefix("Z: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec3(field_name, Vec3 { x, y, z });
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
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::UVec2(field_name, UVec2 { x, y });
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
            if ui
                .add(egui::DragValue::new(&mut y).prefix("Y: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::UVec2(field_name, UVec2 { x, y });
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            };
        });
    }

    pub fn show_string(&mut self, field_name: &'static str, value: String) {
        let id_temp = egui::Id::new((self.selected_entity.id, self.component_name, self.index));
        let mut text = self
            .ui
            .data_mut(|w| w.get_temp::<String>(id_temp))
            .unwrap_or(value.clone());
        let mut response = None;
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            response = Some(ui.text_edit_singleline(&mut text));
        });
        let lost = response.as_ref().is_some_and(|r| r.lost_focus());
        let focused = response.as_ref().is_some_and(|r| r.has_focus());
        if focused {
            self.ui.data_mut(|w| w.insert_temp(id_temp, text.clone()));
        } else {
            if lost {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::String(field_name, text);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
            self.ui.data_mut(|w| w.insert_temp(id_temp, value.clone()));
        }
    }

    pub fn show_name_string(&mut self, field_name: &'static str, value: String) {
        let id_temp = egui::Id::new((
            self.selected_entity.id,
            self.component_name,
            self.index,
            "n",
        ));
        let mut text = self
            .ui
            .data_mut(|w| w.get_temp::<String>(id_temp))
            .unwrap_or(value.clone());
        let response = self.ui.text_edit_singleline(&mut text);
        let lost = response.lost_focus();
        let focused = response.has_focus();
        if focused {
            self.ui.data_mut(|w| w.insert_temp(id_temp, text.clone()));
        } else {
            if lost {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::NameString(field_name, text);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
            self.ui.data_mut(|w| w.insert_temp(id_temp, value.clone()));
        }
    }

    pub fn show_trans_quat(&mut self, field_name: &'static str, value: Quat) {
        let (rx, ry, rz) = value.to_euler(glam::EulerRot::XYZ);
        let mut x = rx.to_degrees();
        let mut y = ry.to_degrees();
        let mut z = rz.to_degrees();

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
            };
        });
    }

    pub fn show_color(&mut self, field_name: &'static str, value: [u8; 4]) {
        let mut color = Color32::from_rgba_unmultiplied(value[0], value[1], value[2], value[3]);
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui.color_edit_button_srgba(&mut color).changed() {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Color(field_name, color.to_array());
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
        });
    }
}
