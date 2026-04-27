use crate::ecs::AnyStorage;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub static COMPONENT_REGISTRY: OnceLock<
    Mutex<HashMap<&'static str, fn(&[u8]) -> Box<dyn AnyStorage>>>,
> = OnceLock::new();

pub fn register_component(name: &'static str, deserialize_fn: fn(&[u8]) -> Box<dyn AnyStorage>) {
    COMPONENT_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(name, deserialize_fn);
}
