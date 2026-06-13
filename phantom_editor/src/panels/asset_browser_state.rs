use std::{fs::DirEntry, path::PathBuf};

use egui::{Response, Ui, include_image};
use phantom_assets::asset_manager::AssetType;
use uuid::Uuid;

use crate::context::EditorContext;

pub struct AssetBrowserState {
    pub grid_length: usize,
    pub button_scale: f32,
    pub max_button_scale: f32,
    pub grid_scaling_offset: f32,
    pub directory_nav_paths: Vec<PathBuf>,
}

impl AssetBrowserState {
    pub fn new() -> Self {
        Self {
            grid_length: 5,
            button_scale: 100.0,
            max_button_scale: 200.0,
            grid_scaling_offset: 0.75,
            directory_nav_paths: Vec::new(),
        }
    }

    pub fn current_dir(&self) -> PathBuf {
        self.directory_nav_paths.last().unwrap().clone()
    }

    pub fn show(&mut self, ui: &mut Ui, ectx: &EditorContext) {
        if self.directory_nav_paths.is_empty() {
            self.directory_nav_paths.push(ectx.project_path.clone());
        }

        let dir = self.directory_nav_paths.last().unwrap();
        let entries: Vec<DirEntry> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();

        let width = ui.available_width();
        let grid_length = (width / self.button_scale) - self.grid_scaling_offset;
        let num_entries = entries.len();
        let num_rows = num_entries.div_ceil(grid_length as usize);

        egui::Panel::top("asset_browser_top_bar").show_inside(ui, |ui| {
            self.show_navigation_bar(ui);
        });

        egui::Panel::bottom("asset_browser_bottom_bar").show_inside(ui, |ui| {
            self.show_file_icon_scale_slider(ui);
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("asset_browser")
                .max_col_width(self.button_scale)
                .spacing([0.0, 0.0])
                .show(ui, |ui| {
                    let mut i = 0;
                    for _row in 0..num_rows {
                        for _col in 0..grid_length as usize {
                            if i >= num_entries {
                                break;
                            }
                            if let Some(response) = self.file_button(ui, &entries, i) {
                                if let Err(e) = self.handle_drag(&response, &entries[i], &ectx) {
                                    log::error!("Failed to set drag payload: {e}");
                                }
                                self.handle_button_response(&response, &entries[i]);
                            }
                            i += 1;
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn file_button(
        &mut self,
        ui: &mut Ui,
        entries: &Vec<DirEntry>,
        index: usize,
    ) -> Option<egui::Response> {
        let file_icon = egui::Image::new(include_image!("../images/folder_icon.png"))
            .fit_to_exact_size(egui::Vec2 {
                x: self.button_scale,
                y: self.button_scale,
            });
        let mut response = None;
        ui.vertical_centered(|ui| {
            response = Some(
                ui.add_sized(
                    [self.button_scale, self.button_scale],
                    egui::Button::image(file_icon.clone())
                        .fill(egui::Color32::TRANSPARENT)
                        .sense(egui::Sense::click_and_drag()),
                ),
            );
            let labels = self.parse_file_names(entries);
            ui.label(format!("{}", labels[index]));
        });
        response
    }

    /// Sets a drag-and-drop payload for `entry` if it is a valid asset.
    ///
    /// Checks the file extension first, if the type is [`AssetType::Invalid`]
    /// the entry is silently skipped. Otherwise the adjacent `.passet` sidecar
    /// is read and the resulting [`Uuid`] and [`AssetType`] are set as the
    /// drag payload for the inspector to consume on drop.
    ///
    /// # Errors
    /// Returns an error if the `.passet` sidecar cannot be read or deserialised.
    fn handle_drag(
        &self,
        response: &Response,
        entry: &DirEntry,
        ectx: &EditorContext,
    ) -> anyhow::Result<()> {
        let asset_path = entry.path();
        let asset_type = ectx.asset_manager.determine_asset_type(asset_path.clone());
        if asset_type == AssetType::Invalid {
            return anyhow::Ok(());
        }

        let payload = ectx.asset_manager.find_uuid_and_asset_type(&asset_path)?;
        response.dnd_set_drag_payload::<(Uuid, AssetType)>(payload);
        Ok(())
    }

    fn handle_button_response(&mut self, response: &Response, entry: &DirEntry) {
        if response.clicked() {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    log::error!("COULD NOT GATHER FILE TYPE {e}");
                    return;
                }
            };

            if file_type.is_dir() {
                self.directory_nav_paths.push(entry.path());
            }
        }
    }

    fn show_navigation_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let paths = self.directory_nav_paths.clone();
            for (index, path) in paths.iter().enumerate() {
                let label = path.file_name().unwrap().to_string_lossy();
                ui.add_space(-8.0);
                let response = ui.add(egui::Button::new(label).fill(egui::Color32::TRANSPARENT));
                ui.add_space(-8.0);
                ui.label("/");
                if response.clicked() {
                    log::debug!("{index}");
                    let last_index = self.directory_nav_paths.len() - 1;
                    log::debug!("{last_index}");
                    if index != last_index {
                        log::debug!("CLICKED");
                        self.directory_nav_paths.truncate(index + 1);
                    }
                }
            }
        });
    }

    fn show_file_icon_scale_slider(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            ui.add(egui::Slider::new(
                &mut self.button_scale,
                1.0..=self.max_button_scale,
            ));
        });
    }

    fn parse_file_names(&mut self, entries: &Vec<DirEntry>) -> Vec<String> {
        let labels: Vec<String> = entries
            .iter()
            .map(|entry| {
                let label = entry.file_name().to_string_lossy().into_owned();
                label.to_string()
            })
            .collect();
        labels
    }
}
