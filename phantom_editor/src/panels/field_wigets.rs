use std::path::PathBuf;

use egui::{Color32, Ui};
use glam::{Quat, UVec2, Vec2, Vec3};
use phantom_assets::asset_manager::{AssetManager, AssetType};
use phantom_core::{
    ecs::{
        Entity, World,
        components::{Animator, Sprite},
    },
    reflecton::{asset_types::SpriteAsset, fields::Field},
};
use uuid::Uuid;

pub struct FieldContext<'a> {
    pub ui: &'a mut Ui,
    pub world: &'a mut World,
    pub component_name: &'a String,
    pub selected_entity: Entity,
    pub fields: &'a Vec<Field>,
    pub index: usize,
}

impl<'a> FieldContext<'a> {
    pub fn show_bool(&mut self, field_name: &'static str, mut value: bool) {
        self.ui.horizontal(|ui| {
            if ui.checkbox(&mut value, field_name).changed() {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Bool(field_name, value);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
        });
    }

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
    pub fn show_i32(&mut self, field_name: &'static str, mut value: i32) {
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut value).speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::I32(field_name, value);
                self.world.set_component_fields(
                    self.component_name.clone(),
                    self.selected_entity,
                    new_fields,
                );
            }
        });
    }
    pub fn show_u32(&mut self, field_name: &'static str, mut value: u32) {
        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut value).speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::U32(field_name, value);
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
    pub fn show_vec2(&mut self, field_name: &'static str, value: Vec2) {
        let mut x = value.x;
        let mut y = value.y;

        self.ui.horizontal(|ui| {
            ui.label(field_name);
            if ui
                .add(egui::DragValue::new(&mut x).prefix("X: ").speed(0.1))
                .changed()
            {
                let mut new_fields = self.fields.clone();
                new_fields[self.index] = Field::Vec2(field_name, Vec2 { x, y });
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
                new_fields[self.index] = Field::Vec2(field_name, Vec2 { x, y });
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
    pub fn show_sprite(
        &mut self,
        asset_manager: &AssetManager,
        field_name: &'static str,
        value: Uuid,
    ) {
        self.ui.horizontal(|ui| {
            ui.label(field_name);

            let frame = egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.inactive.bg_stroke)
                .corner_radius(4.0);

            let display = if value.is_nil() {
                "None (Sprite)".to_string()
            } else {
                asset_manager
                    .find_sprite_by_id(&value)
                    .and_then(|a| {
                        a.get_asset_path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                    })
                    .unwrap_or_else(|| format!("Sprite {}", &value.to_string()[..8]))
            };

            let (_, payload) = ui.dnd_drop_zone::<PathBuf, _>(frame, |ui| {
                ui.label(display).on_hover_text(value.to_string());
            });

            if let Some(payload) = payload {
                let (uuid, asset_type) = match asset_manager.find_uuid_and_asset_type(&payload) {
                    Ok(pair) => pair,
                    Err(e) => {
                        log::warn!("This is not a valid sprite asset {e}");
                        return;
                    }
                };
                if asset_type == AssetType::Sprite {
                    let mut new_fields = self.fields.clone();
                    new_fields[self.index] = Field::Sprite(field_name, uuid);
                    self.world.set_component_fields(
                        self.component_name.clone(),
                        self.selected_entity,
                        new_fields,
                    );
                }
            }
        });
    }

    pub fn show_animator(&mut self, asset_manager: &AssetManager) {
        let Some(anim) = self.world.get_component::<Animator>(self.selected_entity) else {
            return;
        };

        let mut playing = anim.playing;
        let old_current = anim.current;
        let mut current = anim.current;
        let mut sprite_ids = anim.sprite_ids.clone();
        let mut frame_widths = anim.frame_widths.clone();
        let mut frame_heights = anim.frame_heights.clone();
        let mut frame_counts = anim.frame_counts.clone();
        let mut fps_vec = anim.fps.clone();
        let mut looping = anim.looping.clone();

        let mut changed = false;
        let mut remove_idx: Option<usize> = None;

        self.ui.horizontal(|ui| {
            if ui.checkbox(&mut playing, "Playing").changed() {
                changed = true;
            }
            ui.label("Clip:");
            let max = sprite_ids.len().saturating_sub(1);
            if ui
                .add(egui::DragValue::new(&mut current).range(0..=max))
                .changed()
            {
                changed = true;
            }
        });

        for i in 0..sprite_ids.len() {
            self.ui.separator();
            self.ui.horizontal(|ui| {
                ui.label(format!("Clip {i}"));
                if ui.small_button("X").clicked() {
                    remove_idx = Some(i);
                    changed = true;
                }
            });
            self.ui.horizontal(|ui| {
                ui.label("Sprite");
                let frame = egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(ui.visuals().widgets.inactive.bg_stroke)
                    .corner_radius(4.0);
                let display = if sprite_ids[i].is_nil() {
                    "None (Sprite)".to_string()
                } else {
                    asset_manager
                        .find_sprite_by_id(&sprite_ids[i])
                        .and_then(|a| {
                            a.get_asset_path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().into_owned())
                        })
                        .unwrap_or_else(|| "Unknown".to_string())
                };
                let (_, payload) = ui.dnd_drop_zone::<PathBuf, _>(frame, |ui| {
                    ui.label(display);
                });
                if let Some(path) = payload {
                    match asset_manager.find_uuid_and_asset_type(&path) {
                        Ok((uuid, AssetType::Sprite)) => {
                            sprite_ids[i] = uuid;
                            changed = true;
                        }
                        Ok(_) => log::warn!("Dropped asset is not a sprite"),
                        Err(e) => log::warn!("Invalid asset drop: {e}"),
                    }
                }
            });
            self.ui.horizontal(|ui| {
                ui.label("Frame Width");
                if ui.add(egui::DragValue::new(&mut frame_widths[i])).changed() {
                    changed = true;
                }
            });
            self.ui.horizontal(|ui| {
                ui.label("Frame Height");
                if ui
                    .add(egui::DragValue::new(&mut frame_heights[i]))
                    .changed()
                {
                    changed = true;
                }
            });
            self.ui.horizontal(|ui| {
                ui.label("Frame Count");
                if ui
                    .add(egui::DragValue::new(&mut frame_counts[i]).range(1..=u32::MAX))
                    .changed()
                {
                    changed = true;
                }
            });
            self.ui.horizontal(|ui| {
                ui.label("FPS");
                if ui
                    .add(
                        egui::DragValue::new(&mut fps_vec[i])
                            .speed(0.5)
                            .range(0.0..=240.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            self.ui.horizontal(|ui| {
                if ui.checkbox(&mut looping[i], "Loop").changed() {
                    changed = true;
                }
            });
        }

        if self.ui.button("+ Add Clip").clicked() {
            sprite_ids.push(Uuid::nil());
            frame_widths.push(0);
            frame_heights.push(0);
            frame_counts.push(1);
            fps_vec.push(12.0);
            looping.push(true);
            changed = true;
        }

        if changed {
            if let Some(i) = remove_idx {
                if i < sprite_ids.len() {
                    sprite_ids.remove(i);
                }
                if i < frame_widths.len() {
                    frame_widths.remove(i);
                }
                if i < frame_heights.len() {
                    frame_heights.remove(i);
                }
                if i < frame_counts.len() {
                    frame_counts.remove(i);
                }
                if i < fps_vec.len() {
                    fps_vec.remove(i);
                }
                if i < looping.len() {
                    looping.remove(i);
                }
            }

            // Capture the original sprite to restore BEFORE writing the new state,
            // so we can apply it synchronously — avoids a one-frame flash of the
            // raw sprite sheet when Playing is toggled off.
            let orig_to_restore = if !playing {
                self.world
                    .get_component_mut::<Animator>(self.selected_entity)
                    .and_then(|anim| anim.original_sprite.take())
            } else {
                None
            };

            if let Some(anim) = self
                .world
                .get_component_mut::<Animator>(self.selected_entity)
            {
                anim.playing = playing;
                let new_current = current.min(sprite_ids.len().saturating_sub(1));
                if new_current != old_current {
                    anim.frame = 0.0;
                }
                anim.current = new_current;
                anim.sprite_ids = sprite_ids;
                anim.frame_widths = frame_widths;
                anim.frame_heights = frame_heights;
                anim.frame_counts = frame_counts;
                anim.fps = fps_vec;
                anim.looping = looping;
            }

            if let Some(orig) = orig_to_restore {
                if let Some(sprite) = self.world.get_component_mut::<Sprite>(self.selected_entity) {
                    sprite.asset = SpriteAsset(orig);
                }
            }
        }
    }
}
