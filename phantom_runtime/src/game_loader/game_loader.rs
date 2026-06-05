use anyhow::Ok;
use phantom_common::dirs::dirs::PlayerDirs;
use phantom_core::ecs::World;

pub struct GameLoader {}
impl GameLoader {
    pub fn load_world() -> anyhow::Result<World> {
        let data_path = PlayerDirs::data();
        let world_path = std::fs::read_dir(&data_path)?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "pworld")
            })
            .map(|entry| entry.path())
            .ok_or(anyhow::anyhow!("no .pworld file found in data/"))?;
        let world_bytes = std::fs::read(&world_path)?;
        let world = World::deserialize(&world_bytes);
        Ok(world)
    }

    pub fn load_dylib() -> anyhow::Result<libloading::Library> {
        let data_path = PlayerDirs::data();
        let dylib_path = std::fs::read_dir(&data_path)?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "so" || ext == "dll")
            })
            .map(|entry| entry.path())
            .ok_or(anyhow::anyhow!("no dylib found in data/"))?;

        let lib = unsafe { libloading::Library::new(dylib_path)? };
        Ok(lib)
    }

    /// Invokes the game dylib's `phantom_init` function to register its component and script types.
    ///
    /// # Errors
    /// Returns an error if the dylib doesn't export `phantom_init`.
    pub fn init_dylib(lib: &libloading::Library) -> anyhow::Result<()> {
        use phantom_core::ecs::component_registry::get_component_registry_ptr;
        use phantom_core::scripting::script_registry::get_script_registry_ptr;

        log::trace!("Loading phantom_init symbol from dylib");

        unsafe {
            let phantom_init: libloading::Symbol<
                unsafe extern "C" fn(*mut (), *mut (), &'static dyn log::Log, log::LevelFilter),
            > = lib.get(b"phantom_init")?;

            log::trace!("Calling phantom_init with registry pointers");
            // Hand the host's installed logger + level across so the dylib's `log`
            // global points at the same sink (editor console / player stderr).
            phantom_init(
                get_component_registry_ptr() as *mut (),
                get_script_registry_ptr() as *mut (),
                log::logger(),
                log::max_level(),
            );
            log::trace!("phantom_init completed");
        }

        log::trace!("Component and script registration complete");
        Ok(())
    }
}
