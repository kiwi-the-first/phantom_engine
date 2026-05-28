use crate::{
    ecs::World,
    scripting::{ScriptContext, script_registry::SCRIPT_UPDATE_REGISTRY},
};

pub struct ScriptScheduler {}

impl ScriptScheduler {
    pub fn run_all_start_scripts(world: &mut World, ctx: &ScriptContext) {
        let registry = SCRIPT_UPDATE_REGISTRY.get().unwrap().lock().unwrap();
        for (name, (start_fn, _)) in registry.iter() {
            log::trace!("[script_scheduler] Running start for {}", name);
            start_fn(world, ctx);
        }
    }

    pub fn run_all_update_scripts(world: &mut World, ctx: &ScriptContext) {
        let registry = SCRIPT_UPDATE_REGISTRY.get().unwrap().lock().unwrap();
        for (name, (_, update_fn)) in registry.iter() {
            update_fn(world, ctx);
        }
    }
}
