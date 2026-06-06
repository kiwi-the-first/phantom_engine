use crate::ecs::{AnyStorage, Entity, World};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct ComponentEntry(
    pub fn(&[u8]) -> Box<dyn AnyStorage>,          // deserialize_fn
    pub fn(Entity) -> Box<dyn FnOnce(&mut World)>, // add_default_fn
    pub fn(&mut World, Entity),                    // remove_fn
    pub bool,                                      // is_builtin
);

pub static COMPONENT_REGISTRY: OnceLock<Mutex<HashMap<&'static str, ComponentEntry>>> =
    OnceLock::new();

pub fn clear_game_components() {
    let registry = COMPONENT_REGISTRY.get().unwrap();
    let mut guard = registry.lock().unwrap();
    // Only remove game components, keep built-in ones (Transform, Name, Camera, etc)
    guard.retain(|_, entry| entry.3);
}

pub fn get_component_registry_ptr() -> *mut HashMap<&'static str, ComponentEntry> {
    let mutex = COMPONENT_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = match mutex.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("DEADLOCK: Component registry already locked at [location]");
            panic!("Component registry deadlock");
        }
    };

    &mut *guard as *mut _
}

pub fn register_component(name: &'static str, component_entry: ComponentEntry) {
    COMPONENT_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(name, component_entry);
}

pub fn add_component_by_name(world: &mut World, entity: Entity, component_name: &str) {
    let factory = {
        let registry = COMPONENT_REGISTRY.get().unwrap().lock().unwrap();
        registry.get(component_name).unwrap().1
    };

    let closure = factory(entity);
    closure(world);
}

pub fn get_registered_component_names() -> Vec<String> {
    let components = COMPONENT_REGISTRY.get().unwrap().lock().unwrap();
    components.keys().map(|s| s.to_string()).collect()
}

pub fn get_remove_fn(component_name: &str) -> Option<fn(&mut World, Entity)> {
    let registry = COMPONENT_REGISTRY.get()?.lock().unwrap();
    registry.get(component_name).map(|entry| entry.2)
}
