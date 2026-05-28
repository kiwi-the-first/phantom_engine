use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::{ecs::World, scripting::ScriptContext};

pub static SCRIPT_UPDATE_REGISTRY: OnceLock<
    Mutex<
        HashMap<
            &'static str,
            (
                fn(&mut World, &ScriptContext),
                fn(&mut World, &ScriptContext),
            ),
        >,
    >,
> = OnceLock::new();

/// Returns a raw pointer to the script update registry HashMap.
///
/// # Safety
///
/// This function is intended to be called from the dylib's `phantom_init` function.
/// The caller must:
/// - Not hold the pointer after this function's MutexGuard is dropped
/// - Ensure the pointer is only used while the registry mutex is not locked elsewhere
/// - Not cause undefined behavior by creating aliasing mutable references
///
/// The function acquires a lock, extracts a raw pointer, then immediately releases
/// the lock when the guard goes out of scope at the end of this function.
pub fn get_script_registry_ptr() -> *mut HashMap<
    &'static str,
    (
        fn(&mut World, &ScriptContext),
        fn(&mut World, &ScriptContext),
    ),
> {
    let mutex = SCRIPT_UPDATE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = mutex.lock().unwrap();
    &mut *guard as *mut _
}

pub fn clear_all_scripts() {
    let registry = SCRIPT_UPDATE_REGISTRY.get().unwrap();
    let mut guard = registry.lock().unwrap();
    guard.clear(); // Scripts are all from game, clear everything
}

pub fn register_script_functions(
    name: &'static str,
    start_fn: fn(&mut World, &ScriptContext),
    update_fn: fn(&mut World, &ScriptContext),
) {
    SCRIPT_UPDATE_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(name, (start_fn, update_fn));
}
