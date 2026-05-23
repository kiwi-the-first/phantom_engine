use std::path::PathBuf;

pub struct BuildDirs {}
impl BuildDirs {
    /// Returns the project root directory
    /// * `project_path` - Path to the project root
    pub fn root(project_path: &PathBuf) -> PathBuf {
        project_path.clone()
    }
    /// Returns the build output directory → `<project>/build/`
    /// * `project_path` - Path to the project root
    pub fn build(project_path: &PathBuf) -> PathBuf {
        project_path.join("build")
    }
    /// Returns the data directory → `<project>/build/data/`
    /// * `project_path` - Path to the project root
    pub fn data(project_path: &PathBuf) -> PathBuf {
        project_path.join("build").join("data")
    }
    /// Returns the assets directory → `<project>/build/data/assets/`
    /// * `project_path` - Path to the project root
    pub fn assets(project_path: &PathBuf) -> PathBuf {
        project_path.join("build").join("data").join("assets")
    }
}

pub struct PlayerDirs {}
impl PlayerDirs {
    /// Returns the data directory relative to the player executable → `<exe_dir>/data/`
    /// This is where the built world and assets are expected to be found at runtime.
    /// Panics if the executable path cannot be determined.
    pub fn data() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("data")
    }
}

pub struct SystemDirs {}
impl SystemDirs {
    pub fn config() -> Option<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("", "phantom", "phantom");
        proj_dirs.map(|dir| dir.config_dir().to_path_buf())
    }

    pub fn cache() -> Option<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("", "phantom", "phantom");
        proj_dirs.map(|dir| dir.cache_dir().to_path_buf())
    }

    pub fn data() -> Option<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("", "phantom", "phantom");
        proj_dirs.map(|dir| dir.data_dir().to_path_buf())
    }
}
