use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, Result, anyhow};
use libloading::Library;
use phantom_build::BuildSystem;
use phantom_core::{
    ecs::{Entity, World},
    input::{InputSystem, input_system},
    scripting::{ScriptContext, script_scheduler::ScriptScheduler},
    time::time_system::{self, TimeSystem},
};
use phantom_project::{
    phantom_project::PhantomProject, project_manager::project_manager::ProjectManager,
};
use phantom_runtime::{
    asset_manager::asset_manager::{self, AssetManager},
    game_loader::game_loader::GameLoader,
};

use crate::logger::LogEntry;

pub struct EditorContext {
    pub project_path: PathBuf,
    pub project: PhantomProject,
    pub active_world: World,
    pub selected_entity: Option<Entity>,
    pub log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    pub asset_manager: AssetManager,
    pub game_dylib: Option<Library>,
    pub is_playing: bool,
    pub world_snapshot: Option<Vec<u8>>,
    pub input_system: Option<InputSystem>,
    pub time_system: Option<TimeSystem>,
}

impl EditorContext {
    pub fn new(
        project_path: PathBuf,
        project: PhantomProject,
        world: World,
        log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    ) -> Self {
        let asset_manager = AssetManager::default();
        let input_system = InputSystem::default();
        let time_system = TimeSystem::default();
        Self {
            project_path: project_path,
            project: project,
            active_world: world,
            selected_entity: None,
            log_buffer,
            asset_manager,
            game_dylib: None,
            is_playing: false,
            world_snapshot: None,
            input_system: Some(input_system),
            time_system: Some(time_system),
        }
    }

    pub fn load_world(&mut self) -> anyhow::Result<()> {
        let (_, world) = ProjectManager::load(self.project_path.clone())?;
        self.active_world = world;
        Ok(())
    }

    pub fn build_project(&self) {
        if let Err(e) = BuildSystem::build(self.project_path.clone()) {
            log::error!("FAILED TO BUILD GAME! {e}")
        }
    }

    /// Load or reload the game dylib.
    /// `snapshot`: if Some, restore world from these bytes (used during play).
    ///             if None, reload world from disk (used on startup).
    pub fn load_dylib(&mut self, snapshot: Option<Vec<u8>>) -> Result<()> {
        if self.game_dylib.is_some() {
            log::info!("Clearing registries...");
            phantom_core::ecs::component_registry::clear_game_components();
            phantom_core::scripting::script_registry::clear_all_scripts();
        }

        let data_dir = self.project_path.join("build").join("data");

        let dylib_path = std::fs::read_dir(&data_dir)?
            .filter_map(|result| result.ok())
            .find(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                !name.starts_with("_hot_")
                    && (entry
                        .path()
                        .extension()
                        .map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib"))
            })
            .map(|entry| entry.path())
            .ok_or_else(|| anyhow::anyhow!("No dylib found in {:?}", data_dir))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let shadow_path = data_dir.join(format!("_hot_{}.so", timestamp));
        std::fs::copy(&dylib_path, &shadow_path)?;
        log::info!("Shadow copy: {:?}", shadow_path);

        let dylib = unsafe { libloading::Library::new(&shadow_path)? };
        GameLoader::init_dylib(&dylib)?;

        // Drop the old world BEFORE replacing the dylib.
        // The old world's component storage boxes have vtables pointing into the old dylib.
        // If we drop the dylib first, those vtable pointers become dangling → segfault.
        // Assigning World::new() here forces the old world to drop while the old dylib
        // is still loaded and its vtables are still valid.
        self.active_world = World::new();
        self.game_dylib = Some(dylib);
        log::info!("Game dylib loaded!");

        for entry in std::fs::read_dir(&data_dir)?.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("_hot_") && name.ends_with(".so") {
                if entry.path() != shadow_path {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        match snapshot {
            Some(bytes) => {
                log::info!("Restoring world from in-memory snapshot...");
                self.active_world = World::deserialize(&bytes);
            }
            None => {
                log::info!("Reloading world from disk...");
                self.load_world()?;
            }
        }

        Ok(())
    }

    pub fn sync_assets(&mut self) -> Result<()> {
        self.asset_manager
            .load_sprite_assets(&self.active_world, self.project_path.as_path())?;
        Ok(())
    }

    pub fn reload_project(&mut self) -> anyhow::Result<()> {
        log::trace!("Reloading the game dylib");
        let snapshot = self.active_world.serialize();

        // Build game
        self.build_project();

        // Reload dylib, restoring world from in-memory snapshot instead of disk
        if let Err(e) = self.load_dylib(Some(snapshot.clone())) {
            log::error!("Failed to reload dylib: {}", e)
        }
        Ok(())
    }

    pub fn start_playing(&mut self) {
        if self.is_playing {
            return;
        }
        log::trace!("started play!");

        // Serialize FIRST while dylib still loaded and vtables are valid
        let snapshot = self.active_world.serialize();

        // Store snapshot for stop/restore
        self.world_snapshot = Some(snapshot);
        self.is_playing = true;

        if let (Some(input_system), Some(time_system)) = (&self.input_system, &self.time_system) {
            let script_ctx = ScriptContext {
                input: &input_system.input_ctx,
                time: &time_system.time_ctx,
            };
            ScriptScheduler::run_all_start_scripts(&mut self.active_world, &script_ctx);
        }
    }

    pub fn stop_playing(&mut self) {
        if let Some(snapshot) = &self.world_snapshot {
            self.active_world = World::deserialize(snapshot);
        }
        self.is_playing = false;
        self.world_snapshot = None;
    }

    pub fn get_world_and_systems(
        &mut self,
    ) -> Result<(&mut World, &mut InputSystem, &mut TimeSystem)> {
        if let (Some(input), Some(time)) = (self.input_system.as_mut(), self.time_system.as_mut()) {
            Ok((&mut self.active_world, input, time))
        } else {
            Err(anyhow!("FAILED TO FETCH SYSTEMS!"))
        }
    }
}
