use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use egui::{Response, Ui, include_image};
use phantom_assets::asset_manager::{AssetType, consts::PHANTOM_ASSET_EXTENSION};
use uuid::Uuid;

use crate::{
    context::EditorContext,
    panels::asset_browser::{AssetBrowserCRUD, FileButton},
};

pub struct AssetBrowserState {
    pub grid_length: usize,
    pub button_scale: f32,
    pub max_button_scale: f32,
    pub grid_scaling_offset: f32,
    pub directory_nav_paths: Vec<PathBuf>,
    pub directory_file_buttons: Vec<FileButton>,
    entries: Vec<DirEntry>,
    crud: AssetBrowserCRUD,
    should_create_entries: bool,
}

impl AssetBrowserState {
    pub fn new() -> Self {
        Self {
            grid_length: 5,
            button_scale: 100.0,
            max_button_scale: 200.0,
            grid_scaling_offset: 0.75,
            directory_nav_paths: Vec::new(),
            directory_file_buttons: Vec::new(),
            entries: Vec::new(),
            crud: AssetBrowserCRUD::default(),
            should_create_entries: true,
        }
    }

    pub fn current_dir(&self) -> PathBuf {
        self.directory_nav_paths.last().unwrap().clone()
    }

    pub fn show(&mut self, ui: &mut Ui, ectx: &mut EditorContext) {
        if self.directory_nav_paths.is_empty() {
            self.directory_nav_paths.push(ectx.project_path.clone());
        }

        egui::Panel::top("asset_browser_top_bar").show_inside(ui, |ui| {
            self.show_navigation_bar(ui, ectx);
        });

        egui::Panel::bottom("asset_browser_bottom_bar").show_inside(ui, |ui| {
            self.show_file_icon_scale_slider(ui);
        });

        if self.should_create_entries {
            self.entries.clear();
            let dir = self.current_dir();
            self.entries = std::fs::read_dir(dir)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .collect();

            self.entries.retain(|e| {
                !e.path()
                    .extension()
                    .map_or(false, |ext| ext == PHANTOM_ASSET_EXTENSION)
            });

            self.entries.sort_by(|a, b| {
                let a_dir = a.path().is_dir();
                let b_dir = b.path().is_dir();

                b_dir.cmp(&a_dir).then_with(|| {
                    a.file_name()
                        .to_ascii_lowercase()
                        .cmp(&b.file_name().to_ascii_lowercase())
                })
            });

            self.directory_file_buttons.clear();
            for entry in &self.entries {
                log::debug!("{}", self.directory_file_buttons.len());
                let file_button = FileButton::new(entry.path());
                self.directory_file_buttons.push(file_button);
            }
            self.should_create_entries = false;

            if let Some(path) = self.crud.pending_rename.take() {
                if let Some(button) = self
                    .directory_file_buttons
                    .iter_mut()
                    .find(|b| b.get_path() == path)
                {
                    button.start_rename();
                }
            }
        }

        for button in &self.directory_file_buttons {
            if button.is_renamed() {
                self.should_create_entries = true;
            }
        }

        if self.crud.should_refresh {
            self.should_create_entries = true;
            self.crud.should_refresh = false;
        }

        let width = ui.available_width();
        let grid_length = (width / self.button_scale) - self.grid_scaling_offset;
        let num_entries = self.entries.len();
        let num_rows = num_entries.div_ceil(grid_length as usize);

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

                            if let Some(response) = self.directory_file_buttons[i].show(
                                ui,
                                self.button_scale,
                                &mut ectx.asset_manager,
                                &mut self.crud,
                            ) {
                                if let Err(e) = self.handle_drag(&response, &self.entries[i], &ectx)
                                {
                                    log::error!("Failed to set drag payload: {e}");
                                }
                                let entry = &self.entries[i].path();
                                if let Err(e) = self.handle_button_response(&response, entry) {
                                    log::error!("FAILED TO OPEN FILE OR DIRECTORY {e}")
                                }
                                let current_dir = self.current_dir().clone();
                                self.crud.handle_file_context_menu(
                                    &response,
                                    &mut self.directory_file_buttons[i],
                                    current_dir,
                                    &mut ectx.asset_manager,
                                );
                            }
                            i += 1;
                        }
                        ui.end_row();
                    }
                });
        });

        let void_response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("void"),
            egui::Sense::click(),
        );
        let current_dir = self.current_dir();
        void_response.context_menu(|ui| {
            self.crud.standard_context_menu_options(ui, &current_dir);
        });

        if self.crud.pending_new_component.is_some() {
            let mut name = self.crud.pending_new_component.take().unwrap();
            let mut confirm = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("new_component"))
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(200.0);
                    ui.label("Component name:");
                    let resp = ui.text_edit_singleline(&mut name);
                    resp.request_focus();
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        confirm = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        cancel = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() { confirm = true; }
                        if ui.button("Cancel").clicked() { cancel = true; }
                    });
                });
            if confirm && !name.is_empty() {
                let dir = self.current_dir();
                if let Err(e) = self.crud.confirm_create_component(&dir, &name) {
                    log::error!("FAILED TO CREATE COMPONENT {e}");
                }
            } else if !cancel {
                self.crud.pending_new_component = Some(name);
            }
        }

        if self.crud.pending_new_script.is_some() {
            let mut name = self.crud.pending_new_script.take().unwrap();
            let mut confirm = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("new_script"))
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(200.0);
                    ui.label("Script name:");
                    let resp = ui.text_edit_singleline(&mut name);
                    resp.request_focus();
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        confirm = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        cancel = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() { confirm = true; }
                        if ui.button("Cancel").clicked() { cancel = true; }
                    });
                });
            if confirm && !name.is_empty() {
                let dir = self.current_dir();
                if let Err(e) = self.crud.confirm_create_script(&dir, &name) {
                    log::error!("FAILED TO CREATE SCRIPT {e}");
                }
            } else if !cancel {
                self.crud.pending_new_script = Some(name);
            }
        }
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
        let payload = entry.path();
        response.dnd_set_drag_payload::<PathBuf>(payload);
        Ok(())
    }

    fn handle_button_response(
        &mut self,
        response: &Response,
        entry: &PathBuf,
    ) -> anyhow::Result<()> {
        if response.clicked() {
            if entry.is_dir() {
                self.directory_nav_paths.push(entry.to_path_buf());
                self.should_create_entries = true;
            } else if entry.is_file() {
                open::that(entry)?;
            }
        }
        Ok(())
    }

    fn show_navigation_bar(&mut self, ui: &mut Ui, ectx: &mut EditorContext) {
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
                    self.should_create_entries = true;
                }
                self.crud
                    .handle_drop(path, &Some(response), &mut ectx.asset_manager);
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
}
