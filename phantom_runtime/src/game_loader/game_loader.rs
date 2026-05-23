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
}
