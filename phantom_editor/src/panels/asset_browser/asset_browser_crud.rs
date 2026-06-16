use std::{io::Write, path::PathBuf};

const SCRIPT_TEMPLATE: &str = include_str!("../../../templates/script.template");
const COMPONENT_TEMPLATE: &str = include_str!("../../../templates/component.template");

use egui::{Response, Ui};
use phantom_assets::asset_manager::{AssetManager, AssetType};

use crate::panels::asset_browser::FileButton;

#[derive(Default)]
pub struct AssetBrowserCRUD {
    contents: Option<PathBuf>,
    pub should_refresh: bool,
    pub pending_new_script: Option<String>,
    pub pending_new_component: Option<String>,
    pub pending_rename: Option<PathBuf>,
}

impl AssetBrowserCRUD {
    pub fn handle_file_context_menu(
        &mut self,
        response: &Response,
        file_button: &mut FileButton,
        current_dir: PathBuf,
        assert_manager: &mut AssetManager,
    ) {
        response.context_menu(|ui| {
            self.file_context_menu_options(ui, file_button, &current_dir, assert_manager);
            ui.separator();
            self.standard_context_menu_options(ui, &current_dir);
        });
    }

    pub fn standard_context_menu_options(&mut self, ui: &mut Ui, current_dir: &PathBuf) {
        if ui.button("Reveal in File Manager").clicked() {
            if let Err(e) = open::that(current_dir) {
                log::error!("FAILED TO REVEAL IN FILE MANAGER {e}");
            }
        }

        if ui.button("New Folder").clicked() {
            let new_path = Self::unique_folder_path(current_dir);
            if let Err(e) = std::fs::create_dir(&new_path) {
                log::error!("COULD NOT CREATE DIRECTORY {e}");
            } else {
                self.pending_rename = Some(new_path);
                self.should_refresh = true;
            }
        }
        if ui.button("Create New Script").clicked() {
            self.create_script();
        }
        if ui.button("Create New Component").clicked() {
            self.pending_new_component = Some("new_component".to_string());
        }
    }

    fn create_script(&mut self) {
        self.pending_new_script = Some("new_script".to_string());
    }

    fn unique_paste_path(src: &PathBuf, dest_dir: &PathBuf) -> PathBuf {
        let stem = src.file_stem().unwrap_or(src.as_os_str()).to_string_lossy();
        let ext = src.extension().and_then(|e| e.to_str());
        let make = |suffix: &str| match ext {
            Some(e) => dest_dir.join(format!("{}{}.{}", stem, suffix, e)),
            None => dest_dir.join(format!("{}{}", stem, suffix)),
        };
        let base = make("");
        if !base.exists() {
            return base;
        }
        let mut i = 1;
        loop {
            let candidate = make(&format!("_{}", i));
            if !candidate.exists() {
                return candidate;
            }
            i += 1;
        }
    }

    fn unique_folder_path(dir: &PathBuf) -> PathBuf {
        let base = dir.join("New Folder");
        if !base.exists() {
            return base;
        }
        let mut i = 2;
        loop {
            let path = dir.join(format!("New Folder {}", i));
            if !path.exists() {
                return path;
            }
            i += 1;
        }
    }

    pub fn confirm_create_component(&mut self, dir: &PathBuf, name: &str) -> anyhow::Result<()> {
        let struct_name = Self::to_pascal_case(name);
        let content = COMPONENT_TEMPLATE.replace("{NAME}", &struct_name);
        std::fs::write(dir.join(format!("{}.rs", name)), content)?;
        self.should_refresh = true;
        Ok(())
    }

    pub fn confirm_create_script(&mut self, dir: &PathBuf, name: &str) -> anyhow::Result<()> {
        let struct_name = Self::to_pascal_case(name);
        let content = SCRIPT_TEMPLATE
            .replace("{NAME}", &struct_name)
            .replace("{Name}", &struct_name);
        std::fs::write(dir.join(format!("{}.rs", name)), content)?;
        self.should_refresh = true;
        Ok(())
    }

    fn to_pascal_case(name: &str) -> String {
        name.split('_')
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    fn file_context_menu_options(
        &mut self,
        ui: &mut Ui,
        file_button: &mut FileButton,
        current_dir: &PathBuf,
        assert_manager: &mut AssetManager,
    ) {
        // if ui.button("Cut").clicked() {
        //     // Moooooove
        // }
        if ui.button("Copy").clicked() {
            self.contents = Some(file_button.get_path());
        }
        if self.contents.is_some() {
            if ui.button("Paste").clicked() {
                let src = self.contents.clone().unwrap();
                let dest = Self::unique_paste_path(&src, current_dir);
                if src.is_dir() {
                    if let Err(e) = assert_manager.copy_dir_to(&src, &dest) {
                        log::error!("FAILED TO COPY DIRECTORY {e}");
                    }
                } else {
                    if let Err(e) = std::fs::copy(&src, &dest) {
                        log::error!("FAILED TO COPY FILE {e}");
                    } else {
                        let _ = assert_manager.create_passet_file(&dest);
                    }
                }
                self.should_refresh = true;
            }
        }

        // if ui.button("Copy Location").clicked() {
        // TODO
        // }
        if ui.button("Rename").clicked() {
            file_button.start_rename();
        }
        if ui.button("Delete").clicked() {
            if let Err(e) = self.delete(&file_button.get_path(), assert_manager) {
                log::error!("FAILED TO DELETE {e}");
            }
        }
    }

    pub fn handle_drop(
        &mut self,
        dest: &PathBuf,
        response: &Option<Response>,
        asset_manager: &mut AssetManager,
    ) -> anyhow::Result<()> {
        if response.is_none() {
            return Ok(());
        }
        if let Some(payload) = response.as_ref().unwrap().dnd_release_payload::<PathBuf>() {
            if dest.is_dir() {
                let dropped_path = payload.as_path();
                match asset_manager.determine_asset_type(dropped_path.to_path_buf()) {
                    AssetType::Invalid => {
                        self.move_file(&dropped_path.to_path_buf(), &dest)?;
                        let new_dir = dest.join(dropped_path.file_name().unwrap());
                        if new_dir.is_dir() {
                            asset_manager.update_passets_in_moved_dir(
                                &new_dir,
                                dropped_path,
                                &new_dir,
                            )?;
                        }
                    }
                    _ => {
                        self.move_file(&dropped_path.to_path_buf(), &dest)?;
                        let passet_path = AssetManager::passet_path_for(dropped_path);
                        let mut passet = AssetManager::deserialize(passet_path.clone())?;
                        passet.set_asset_path(dest.join(format!(
                            "{}",
                            dropped_path.file_name().unwrap().to_string_lossy()
                        )));
                        let json = AssetManager::serialize(&passet)?;
                        let mut file = std::fs::File::create(passet_path.clone())?;
                        file.write_all(&json)?;
                        self.move_file(&passet_path, &dest)?;
                        asset_manager.update_asset(
                            &passet.get_id(),
                            &passet.get_asset_type(),
                            passet,
                        );
                    }
                }
                self.should_refresh = true;
            }
        }
        Ok(())
    }

    pub fn delete(
        &mut self,
        path: &PathBuf,
        asset_manager: &mut AssetManager,
    ) -> anyhow::Result<()> {
        if path.is_dir() {
            asset_manager.remove_assets_in_dir(path)?;
            std::fs::remove_dir_all(path)?;
        } else {
            if let Ok((uuid, asset_type)) = asset_manager.find_uuid_and_asset_type(path) {
                asset_manager.remove_asset(&uuid, &asset_type);
            }
            let passet_path = AssetManager::passet_path_for(path);
            if passet_path.exists() {
                std::fs::remove_file(&passet_path)?;
            }
            std::fs::remove_file(path)?;
        }
        self.should_refresh = true;
        Ok(())
    }

    fn move_file(&self, src: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
        let file_name = src.file_name().unwrap().to_string_lossy();
        let new_path = dest.join(format!("{}", file_name));
        std::fs::rename(&src, new_path)?;
        Ok(())
    }
}
