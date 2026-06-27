use std::{
    clone,
    collections::HashMap,
    fs::{DirEntry, read},
    io::Write,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::asset_manager::{
    PhantomAsset, asset_types::AssetType, consts::PHANTOM_ASSET_EXTENSION, phantom_asset,
};

pub struct AssetManager {
    asset_storage_sprite: HashMap<uuid::Uuid, PhantomAsset>,
    asset_storage_audio: HashMap<uuid::Uuid, PhantomAsset>,
    project_root: Option<PathBuf>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            asset_storage_sprite: HashMap::new(),
            asset_storage_audio: HashMap::new(),
            project_root: None,
        }
    }
}

impl AssetManager {
    /// Returns the path `abs` expressed relative to the project root.
    /// If `abs` does not start with the project root it is returned as-is.
    fn to_relative(&self, abs: &Path) -> PathBuf {
        match &self.project_root {
            Some(root) => abs.strip_prefix(root).unwrap_or(abs).to_path_buf(),
            None => abs.to_path_buf(),
        }
    }

    /// Resolves a potentially-relative path against the project root.
    /// If `path` is already absolute it is returned as-is.
    fn resolve(&self, path: &Path) -> PathBuf {
        match &self.project_root {
            Some(root) if path.is_relative() => root.join(path),
            _ => path.to_path_buf(),
        }
    }

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
        self.project_root = Some(project_root.clone());
        self.rebuild_sidecars(project_root, project_root)?;
        Ok(())
    }

    /// Walks the project tree and rebuilds every `.passet` sidecar from scratch.
    ///
    /// For each asset file found, if a sidecar already exists its UUID is
    /// preserved so that world component references remain valid. The sidecar
    /// is then rewritten with the correct absolute path for the current machine.
    /// Sidecars with no matching asset file are deleted.
    fn rebuild_sidecars(&mut self, dir: &PathBuf, root: &PathBuf) -> anyhow::Result<()> {
        let entries: Vec<DirEntry> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                if path == root.join("build") {
                    continue;
                }
                self.rebuild_sidecars(&path, root)?;
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str());

            // Delete sidecars whose asset file no longer sits beside them.
            // We derive the asset path from the sidecar filename rather than
            // trusting the stored path, which may be from another machine.
            if ext == Some(PHANTOM_ASSET_EXTENSION) {
                let asset_path = path.with_extension("");
                if !asset_path.exists() {
                    log::trace!("Removing orphaned sidecar {}", path.to_string_lossy());
                    let _ = std::fs::remove_file(&path);
                }
                continue;
            }

            let asset_type = self.determine_asset_type(path.clone());
            if asset_type == AssetType::Invalid {
                continue;
            }

            let sidecar_path = AssetManager::passet_path_for(&path);

            // Reuse the existing UUID if a sidecar is present so world
            // references aren't broken when the project moves machines.
            let uuid = if sidecar_path.exists() {
                AssetManager::deserialize(sidecar_path.clone())
                    .map(|p| p.get_id())
                    .unwrap_or_else(|_| Uuid::new_v4())
            } else {
                Uuid::new_v4()
            };

            let passet = PhantomAsset::new(uuid, asset_type, self.to_relative(&path));
            let json = serde_json::to_vec(&passet)?;
            std::fs::write(&sidecar_path, json)?;

            log::trace!("Rebuilt sidecar for {}", path.to_string_lossy());

            match asset_type {
                AssetType::Sprite => { self.asset_storage_sprite.insert(uuid, passet); }
                AssetType::Audio  => { self.asset_storage_audio.insert(uuid, passet); }
                _ => {}
            }
        }

        Ok(())
    }

    /// Returns the sidecar path for `asset_path` by appending `.passet` to the
    /// full file name, e.g. `player.png` -> `player.png.passet`.
    ///
    /// The full name is used (not the stem) so that `player.png` and
    /// `player.wav` in the same directory don't collide on one sidecar.
    pub fn passet_path_for(asset_path: &Path) -> PathBuf {
        let mut path = asset_path.as_os_str().to_owned();
        path.push(".");
        path.push(PHANTOM_ASSET_EXTENSION);
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
        let file = read(AssetManager::passet_path_for(src))?;
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

    pub fn update_passets_in_moved_dir(
        &mut self,
        dir: &Path,
        _old_base: &Path,
        _new_base: &Path,
    ) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.update_passets_in_moved_dir(&path, _old_base, _new_base)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some(PHANTOM_ASSET_EXTENSION) {
                let mut passet = AssetManager::deserialize(path.clone())?;
                passet.set_asset_path(self.to_relative(&path.with_extension("")));
                let json = AssetManager::serialize(&passet)?;
                let mut file = std::fs::File::create(&path)?;
                file.write_all(&json)?;
                self.update_asset(&passet.get_id(), &passet.get_asset_type(), passet);
            }
        }
        Ok(())
    }

    pub fn remove_asset(&mut self, uuid: &Uuid, asset_type: &AssetType) {
        match asset_type {
            AssetType::Sprite => { self.asset_storage_sprite.remove(uuid); }
            AssetType::Audio => { self.asset_storage_audio.remove(uuid); }
            _ => {}
        }
    }

    pub fn remove_assets_in_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.remove_assets_in_dir(&path)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some(PHANTOM_ASSET_EXTENSION) {
                let passet = AssetManager::deserialize(path)?;
                self.remove_asset(&passet.get_id(), &passet.get_asset_type());
            }
        }
        Ok(())
    }

    pub fn copy_dir_to(&mut self, src: &Path, dest: &Path) -> anyhow::Result<()> {
        self.copy_dir_recursive(&src.to_path_buf(), &dest.to_path_buf())?;
        self.scan_for_new_assets(&dest.to_path_buf())?;
        Ok(())
    }

    pub fn update_asset(&mut self, uuid: &Uuid, asset_type: &AssetType, passet: PhantomAsset) {
        match asset_type {
            AssetType::Sprite => {
                self.asset_storage_sprite.insert(*uuid, passet);
            }
            AssetType::Audio => {
                self.asset_storage_audio.insert(*uuid, passet);
            }
            _ => {}
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

    /// Deserializes a path to a `.passet`
    ///
    /// # Errors
    /// Returns an error if file cannot be read or json cannot be deserialized.
    pub fn deserialize(passet_path: PathBuf) -> anyhow::Result<PhantomAsset> {
        let file = std::fs::read(passet_path)?;
        let passet: PhantomAsset = serde_json::from_slice(&file)?;
        Ok(passet)
    }

    pub fn serialize(passet: &PhantomAsset) -> anyhow::Result<Vec<u8>> {
        let json = serde_json::to_vec(passet)?;
        Ok(json)
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
                    == Some(PHANTOM_ASSET_EXTENSION)
            {
                log::trace!(
                    "Found {} ... importing",
                    entry.file_name().to_string_lossy()
                );
                let file = read(entry.path())?;
                let mut passet: PhantomAsset = serde_json::from_slice(&file)?;
                if passet.get_asset_path().is_absolute() {
                    passet.set_asset_path(self.to_relative(&passet.get_asset_path()));
                }
                // Migrate legacy stem-named sidecars (`player.passet`) to the
                // full-name convention (`player.png.passet`).
                let expected_path = AssetManager::passet_path_for(&self.resolve(&passet.get_asset_path()));
                if entry.path() != expected_path && self.resolve(&passet.get_asset_path()).is_file() {
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
    pub fn create_passet_file(
        &self,
        file: &PathBuf,
    ) -> anyhow::Result<Option<(PhantomAsset, AssetType)>> {
        if file.extension().and_then(|e| e.to_str()) == Some(PHANTOM_ASSET_EXTENSION) {
            return anyhow::Ok(None);
        }

        let asset_type = self.determine_asset_type(file.clone());
        if asset_type == AssetType::Invalid {
            return anyhow::Ok(None);
        }

        let file_path = AssetManager::passet_path_for(file);

        let passet = PhantomAsset::new(Uuid::new_v4(), asset_type, self.to_relative(file));
        let json = serde_json::to_vec(&passet)?;
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&json)?;
        anyhow::Ok(Some((passet, asset_type)))
    }

    pub fn resolve_asset_path(&self, asset_path: &Path) -> PathBuf {
        self.resolve(asset_path)
    }
}
