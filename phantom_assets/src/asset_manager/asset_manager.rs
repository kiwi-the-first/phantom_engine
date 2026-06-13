use std::{
    clone,
    collections::HashMap,
    fs::{DirEntry, read},
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;
use uuid::Uuid;

use crate::asset_manager::{PhantomAsset, asset_types::AssetType, phantom_asset};

pub struct AssetManager {
    pub phantom_asset_extension: &'static str,
    asset_storage_sprite: HashMap<uuid::Uuid, PhantomAsset>,
    asset_storage_audio: HashMap<uuid::Uuid, PhantomAsset>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            phantom_asset_extension: "passet",
            asset_storage_sprite: HashMap::new(),
            asset_storage_audio: HashMap::new(),
        }
    }
}

impl AssetManager {
    /// Imports a file or directory into the project at `current_open_dir`.
    ///
    /// If `src` is a file, it is copied and a `.passet` metadata sidecar
    /// is generated beside it. If `src` is a directory, the entire tree
    /// is copied recursively and a `.passet` file is created for every asset
    /// found inside.
    ///
    /// # Arguments
    /// - `src` — the source file or folder to import (outside the project)
    /// - `dest` — the destination folder inside the project
    ///
    /// # Errors
    /// Returns an error if:
    /// - `src` is neither a file nor a directory
    /// - any file copy or directory creation fails
    /// - a `.passet` sidecar file cannot be written
    pub fn import_asset(&mut self, src: PathBuf, dest: PathBuf) -> anyhow::Result<()> {
        let file_name = src.file_name().unwrap().to_string_lossy().to_string();
        let dest = dest.join(&file_name);

        if src.is_dir() {
            log::trace!("Importing directory: {}", src.to_string_lossy());
            self.copy_dir_recursive(&src, &dest)?;
            self.scan_for_new_assets(&dest)?;
        } else if src.is_file() {
            log::trace!("Importing file: {}", src.to_string_lossy());
            std::fs::copy(&src, &dest)?;
            if let Some((passet, asset_type)) = self.create_passet_file(&dest)? {
                match asset_type {
                    AssetType::Sprite => {
                        self.asset_storage_sprite.insert(passet.get_id(), passet);
                    }
                    AssetType::Audio => {
                        self.asset_storage_audio.insert(passet.get_id(), passet);
                    }
                    _ => {}
                }
            };
        } else {
            anyhow::bail!(
                "Cannot import '{}': path is not a file or directory",
                src.to_string_lossy()
            );
        }

        log::trace!(
            "Copied {} -> {}",
            src.to_string_lossy(),
            dest.to_string_lossy()
        );

        Ok(())
    }

    /// Initialises the asset manager by scanning the project directory for
    /// existing `.passet` sidecar files and loading them into `asset_storage`.
    ///
    /// # Arguments
    /// - `project_root` — the root directory of the open project
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or a `.passet` file
    /// cannot be deserialised.
    pub fn init(&mut self, project_root: &PathBuf) -> anyhow::Result<()> {
        self.scan_for_passet_and_import(project_root, project_root)?;

        Ok(())
    }

    /// Returns the sidecar path for `asset_path` by appending `.passet` to the
    /// full file name, e.g. `player.png` -> `player.png.passet`.
    ///
    /// The full name is used (not the stem) so that `player.png` and
    /// `player.wav` in the same directory don't collide on one sidecar.
    pub fn passet_path_for(&self, asset_path: &Path) -> PathBuf {
        let mut path = asset_path.as_os_str().to_owned();
        path.push(".");
        path.push(self.phantom_asset_extension);
        PathBuf::from(path)
    }

    /// Reads the `.passet` sidecar adjacent to `src` and returns its UUID and
    /// [`AssetType`].
    ///
    /// The sidecar is expected to live in the same directory as `src` with the
    /// same file name plus a `.passet` extension.
    ///
    /// # Errors
    /// Returns an error if the sidecar file cannot be read or deserialised.
    pub fn find_uuid_and_asset_type(&self, src: &PathBuf) -> anyhow::Result<(Uuid, AssetType)> {
        let file = read(self.passet_path_for(src))?;
        let passet: PhantomAsset = serde_json::from_slice(&file)?;
        let uuid = passet.get_id();
        let asset_type = passet.get_asset_type();
        anyhow::Ok((uuid, asset_type))
    }

    pub fn refresh() -> anyhow::Result<()> {
        // 1. Watch for file changes like Delete Rename and Move And Update passet Files accordingly
        // 2 . Watch for files that could be imported and show file dialog asking to import
        todo!()
    }

    /// Maps a file extension to an [`AssetType`].
    ///
    /// | Extensions                              | Type              |
    /// |-----------------------------------------|-------------------|
    /// | `png`, `jpg`, `jpeg`                    | `AssetType::Sprite` |
    /// | `mp3`, `wav`, `flac`, `ogg`, `mp4`, `m4a`, `aac` | `AssetType::Audio` |
    /// | anything else                           | `AssetType::Invalid` |
    pub fn determine_asset_type(&self, asset_path: PathBuf) -> AssetType {
        match asset_path.extension().and_then(|e| e.to_str()) {
            //Image
            Some("png" | "jpg" | "jpeg") => AssetType::Sprite,
            // Audio (all formats supported by rodio's default Symphonia backend)
            Some("mp3" | "wav" | "flac" | "ogg" | "mp4" | "m4a" | "aac") => AssetType::Audio,
            _ => AssetType::Invalid,
        }
    }
    /// Returns all sprite assets registered in [`AssetManager`]
    pub fn grab_all_registered_sprite_assets(&mut self) -> Vec<&mut PhantomAsset> {
        self.asset_storage_sprite.values_mut().collect()
    }
    /// Returns all audio assets registered in [`AssetManager`]
    pub fn grab_all_registered_audio_assets(&mut self) -> Vec<&mut PhantomAsset> {
        self.asset_storage_audio.values_mut().collect()
    }

    pub fn find_sprite_by_id(&self, uuid: &Uuid) -> Option<&PhantomAsset> {
        self.asset_storage_sprite.get(uuid)
    }

    /// Recursively copies `src` into `dest`, mirroring the directory tree.
    ///
    /// Symlinks are treated as regular files and copied by content.
    /// `dest` is created if it does not already exist.
    ///
    /// # Errors
    /// Returns an error if any read, write, or directory-creation operation fails.
    fn copy_dir_recursive(&self, src: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dest.join(entry.file_name());
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else if file_type.is_file() || file_type.is_symlink() {
                std::fs::copy(&src_path, &dst_path)?;
                log::debug!("  copied {}", src_path.to_string_lossy());
            }
        }
        Ok(())
    }

    /// Recursively walks `dir` and deserialises every `.passet` sidecar file
    /// found into a [`PhantomAsset`], inserting it into `asset_storage`.
    ///
    /// Unlike [`scan_for_new_assets`], this function does **not** create new
    /// sidecar files — it only loads ones that already exist on disk.
    ///
    /// # Errors
    /// Returns an error if any directory entry cannot be read or a `.passet`
    /// file cannot be deserialised.
    fn scan_for_passet_and_import(&mut self, dir: &PathBuf, root: &PathBuf) -> anyhow::Result<()> {
        let entries: Vec<DirEntry> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        for entry in entries {
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if entry.path() == root.join("build") {
                    continue;
                }
                self.scan_for_passet_and_import(&entry.path(), root)?;
            } else if file_type.is_file()
                && entry.path().extension().and_then(|e| e.to_str())
                    == Some(self.phantom_asset_extension)
            {
                log::trace!(
                    "Found {} ... importing",
                    entry.file_name().to_string_lossy()
                );
                let file = read(entry.path())?;
                let mut passet: PhantomAsset = serde_json::from_slice(&file)?;
                if passet.get_asset_path().is_relative() {
                    passet.set_asset_path(root.join(passet.get_asset_path()));
                }
                // Migrate legacy stem-named sidecars (`player.passet`) to the
                // full-name convention (`player.png.passet`).
                let expected_path = self.passet_path_for(&passet.get_asset_path());
                if entry.path() != expected_path && passet.get_asset_path().is_file() {
                    log::info!(
                        "Migrating sidecar {} -> {}",
                        entry.path().to_string_lossy(),
                        expected_path.to_string_lossy()
                    );
                    std::fs::rename(entry.path(), &expected_path)?;
                }
                match passet.get_asset_type() {
                    AssetType::Sprite => {
                        self.asset_storage_sprite.insert(passet.get_id(), passet);
                    }
                    AssetType::Audio => {
                        self.asset_storage_audio.insert(passet.get_id(), passet);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Walks `dir` and registers every non-`.passet` file as a new asset.
    ///
    /// For each file found, a `.passet` sidecar is created and the resulting
    /// [`PhantomAsset`] is inserted into `asset_storage`. Subdirectories are
    /// visited recursively.
    ///
    /// # Errors
    /// Returns an error if a `.passet` file cannot be created or written.
    fn scan_for_new_assets(&mut self, dir: &PathBuf) -> anyhow::Result<()> {
        let entries: Vec<DirEntry> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        for entry in entries {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    log::error!("COULD NOT GATHER FILE TYPE {e}");
                    continue;
                }
            };
            if file_type.is_dir() {
                self.scan_for_new_assets(&entry.path())?;
            }
            if file_type.is_file() {
                let file_path = entry.path();
                if let Some((passet, asset_type)) = self.create_passet_file(&file_path)? {
                    match asset_type {
                        AssetType::Sprite => {
                            self.asset_storage_sprite.insert(passet.get_id(), passet);
                        }
                        AssetType::Audio => {
                            self.asset_storage_audio.insert(passet.get_id(), passet);
                        }
                        _ => {}
                    }
                };
                log::debug!("FOUND FILE {}", entry.file_name().to_string_lossy())
            }
        }
        Ok(())
    }

    /// Creates a `.passet` sidecar file next to `file` and returns the asset.
    ///
    /// The sidecar is a JSON file that stores the asset's UUID, type, and
    /// source path. If `file` is already a `.passet` file, this is a no-op
    /// and `Ok(None)` is returned.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created or the JSON cannot be serialised.
    fn create_passet_file(
        &self,
        file: &PathBuf,
    ) -> anyhow::Result<Option<(PhantomAsset, AssetType)>> {
        if file.extension().and_then(|e| e.to_str()) == Some(self.phantom_asset_extension) {
            return anyhow::Ok(None);
        }

        let asset_type = self.determine_asset_type(file.clone());
        if asset_type == AssetType::Invalid {
            return anyhow::Ok(None);
        }

        let file_path = self.passet_path_for(file);

        let passet = PhantomAsset::new(Uuid::new_v4(), asset_type, file.clone());
        let json = serde_json::to_vec(&passet)?;
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&json)?;
        anyhow::Ok(Some((passet, asset_type)))
    }
}
