use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{SparseSet, components::Transform};

pub struct World {
    sparse_set_storage: HashMap<TypeId, Box<dyn Any>>,
    next_id: u32,
    deleted_ids: Vec<u32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            sparse_set_storage: HashMap::new(),
            next_id: 0,
            deleted_ids: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> u32 {
        // Make sure sparse set for transforms exist
        self.sparse_set_storage
            .entry(TypeId::of::<Transform>())
            .or_insert_with(|| Box::new(SparseSet::<Transform>::new()));
        // generate entity_id
        let entity_id = self.deleted_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id = self.next_id + 1;
            id
        });

        // TODO: give entity a transform immediately

        entity_id
    }
    // pub fn destroy(entity_id) {}
    // pub fn add_component<C>(entity_id, component){}
    // pub fn remove_component<C>(entity_id, component){}
    // pub fn get_component<C>(entity_id, component) -> Option<&C>{}
    // pub fn get_component_mut<C>(entity_id, component) Option<&mut C> {}
    // pub fn query_with<C>() -> Vec<entity_id> {}
    // pub fn query_with2<A,B>() -> Vec<entity> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_spawn() {
        //TODO: test spawning
    }
}
