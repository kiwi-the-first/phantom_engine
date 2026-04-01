use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct World {
    sparse_set_storage: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            sparse_set_storage: HashMap::new(),
        }
    }

    // pub fn spawn() -> entity_id {}
    // pub fn destroy(entity_id) {}
    // pub fn add_component<C>(entity_id, component){}
    // pub fn remove_component<C>(entity_id, component){}
    // pub fn get_component<C>(entity_id, component) -> Option<&C>{}
    // pub fn get_component_mut<C>(entity_id, component) Option<&mut C> {}
    // pub fn query_with<C>() -> Vec<entity_id> {}
    // pub fn query_with2<A,B>() -> Vec<entity> {}
}
