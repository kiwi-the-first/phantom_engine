use crate::ecs::{AnyStorage, Entity, World};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub static COMPONENT_REGISTRY: OnceLock<
    Mutex<
        HashMap<
            &'static str,
            (
                fn(&[u8]) -> Box<dyn AnyStorage>,
                fn(Entity) -> Box<dyn FnOnce(&mut World)>,
            ),
        >,
    >,
> = OnceLock::new();

pub fn register_component(
    name: &'static str,
    deserialize_fn: fn(&[u8]) -> Box<dyn AnyStorage>,
    add_default_fn: fn(Entity) -> Box<dyn FnOnce(&mut World)>,
) {
    COMPONENT_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(name, (deserialize_fn, add_default_fn));
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
