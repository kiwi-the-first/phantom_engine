use std::{
    any,
    fs::DirEntry,
    io::{SeekFrom::Current, Write},
    path::PathBuf,
};

use egui::{Response, Ui, include_image, response};
use phantom_assets::asset_manager::{AssetManager, AssetType, asset_manager};

use crate::panels::asset_browser::{AssetBrowserCRUD, asset_browser_crud};

pub struct FileButton {
    file_path: PathBuf,
    name: String,
    is_renaming: bool,
    shoud_refresh: bool,
}

impl FileButton {
    pub fn new(file_path: PathBuf) -> Self {
        let name = FileButton::parse_file_name(&file_path);
        Self {
            file_path,
            name: name,
            is_renaming: false,
            shoud_refresh: false,
        }
    }
    /// Creates a button for a corresponding file or directory
    ///
    /// # TEMP
    /// Uses a hardcoded path to a folder icon for every file type `../images/folder_icon.png`
    pub fn show(
        &mut self,
        ui: &mut Ui,
        scale: f32,
        asset_manager: &mut AssetManager,
        asset_browser_crud: &mut AssetBrowserCRUD,
    ) -> Option<egui::Response> {
        let file_icon = egui::Image::new(include_image!("../../images/folder_icon.png"))
            .fit_to_exact_size(egui::Vec2 { x: scale, y: scale });

        let mut response = None;
        ui.vertical_centered(|ui| {
            response = Some(
                ui.add_sized(
                    [scale, scale],
                    egui::Button::image(file_icon.clone())
                        .fill(egui::Color32::TRANSPARENT)
                        .sense(egui::Sense::click_and_drag()),
                ),
            );
            self.show_file_name(ui, asset_manager);
        });
        if let Err(e) = asset_browser_crud.handle_drop(&self.file_path, &response, asset_manager) {
            log::error!("UNABLE TO MOVE FILE {e}");
        }
        response
    }

    pub fn start_rename(&mut self) {
        self.is_renaming = true;
    }

    pub fn is_renamed(&self) -> bool {
        self.shoud_refresh
    }

    pub fn get_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn show_file_name(&mut self, ui: &mut Ui, asset_manager: &mut AssetManager) {
        if !self.is_renaming {
            ui.label(format!("{}", self.name));
            return;
        }
        let response = ui.text_edit_singleline(&mut self.name);
        // if !response.has_focus() {
        //     log::debug!("")
        //     response.request_focus();
        // }

        if response.lost_focus() {
            if let Err(e) = self.rename_file(asset_manager) {
                log::error!("UNABLE TO RENAME FILE! {e}")
            }

            self.is_renaming = false;
            self.shoud_refresh = true;
        }
    }

    fn rename_file(&mut self, asset_manager: &mut AssetManager) -> anyhow::Result<()> {
        let file_path = self.file_path.clone();
        let ext = file_path.extension().and_then(|e| e.to_str());

        let parent = file_path.parent().unwrap();
        let name_path = PathBuf::from(self.name.trim());
        let name = match name_path.file_stem() {
            Some(s) if !s.is_empty() => s.to_string_lossy(),
            _ => anyhow::bail!("name cannot be empty"),
        };

        let new_path = match ext {
            Some(ext) => parent.join(format!("{}.{}", name, ext)),
            None => parent.join(name.as_ref()),
        };

        std::fs::rename(file_path.clone(), &new_path)?;
        self.file_path = new_path.clone();

        let Some(ext) = ext else {
            asset_manager.update_passets_in_moved_dir(&new_path, &file_path, &new_path)?;
            return Ok(());
        };

        // Adjust Sidecar
        let passet_path = AssetManager::passet_path_for(file_path.as_path());
        let mut passet = AssetManager::deserialize(passet_path.clone())?;
        passet.set_asset_path(new_path);
        let json = AssetManager::serialize(&passet)?;

        let passet_ext = passet_path.extension().unwrap().to_str().unwrap();
        let parent = file_path.parent().unwrap();
        let new_passet_path = parent.join(format!("{}.{}.{}", name, ext, passet_ext));
        std::fs::rename(passet_path, new_passet_path.clone())?;

        // Update asset manager
        let uuid = passet.get_id();
        let asset_type = passet.get_asset_type();
        asset_manager.update_asset(&uuid, &asset_type, passet);

        let mut file = std::fs::File::create(new_passet_path)?;
        file.write_all(&json)?;

        Ok(())
    }

    fn parse_file_name(entry: &PathBuf) -> String {
        let label = entry.file_name().unwrap().to_string_lossy().into_owned();
        label.to_string();
        label
    }
}
