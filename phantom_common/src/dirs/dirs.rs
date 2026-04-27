use std::path::PathBuf;

use directories::ProjectDirs;

pub fn config() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "phantom", "phantom");
    proj_dirs.map(|dir| dir.config_dir().to_path_buf())
}

pub fn cache() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "phantom", "phantom");
    proj_dirs.map(|dir| dir.cache_dir().to_path_buf())
}

pub fn data() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "phantom", "phantom");
    proj_dirs.map(|dir| dir.data_dir().to_path_buf())
}
