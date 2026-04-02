use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{SparseSet, components::Transform, sparse_set};

pub struct World {
    sparse_set_storage: HashMap<TypeId, Box<dyn Any>>,
    next_entity_id: u32,
    deleted_entity_ids: Vec<u32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            sparse_set_storage: HashMap::new(),
            next_entity_id: 0,
            deleted_entity_ids: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> u32 {
        // Make sure sparse set for transforms exist if not create it
        self.sparse_set_storage
            .entry(TypeId::of::<Transform>())
            .or_insert_with(|| Box::new(SparseSet::<Transform>::new()));

        // Generate entity_id
        let entity_id = self.deleted_entity_ids.pop().unwrap_or_else(|| {
            let id = self.next_entity_id;
            self.next_entity_id = self.next_entity_id + 1;
            id
        });
        // Add transform by default
        self.add_component(entity_id, Transform::default());

        entity_id
    }

    // pub fn destroy(entity_id) {}

    pub fn add_component<C: Any + 'static>(&mut self, entity_id: u32, component: C) {
        self.sparse_set_storage
            .entry(TypeId::of::<C>())
            .or_insert_with(|| Box::new(SparseSet::<C>::new()));

        let sparse_set = self
            .sparse_set_storage
            .get_mut(&TypeId::of::<C>())
            .expect("Component sparse set not found")
            .downcast_mut::<SparseSet<C>>()
            .expect("Downcast failed");

        sparse_set.insert(entity_id, component);
    }
    // pub fn has_component<C>(entity_id, component) -> bool {}
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
    fn check_spawn_generates_transform() {
        let mut world = World::new();
        world.spawn();

        assert_eq!(
            world
                .sparse_set_storage
                .contains_key(&TypeId::of::<Transform>()),
            true
        );
    }

    #[test]
    fn check_spawn_generates_correct_id() {
        let mut world = World::new();
        let entity_zero = world.spawn();
        let entity_one = world.spawn();
        assert_eq!(entity_zero, 0);
        assert_eq!(entity_one, 1);
    }

    #[test]
    fn check_spawned_entity_has_transform() {
        let mut world = World::new();
        let entity = world.spawn();
        let sparse_set = world
            .sparse_set_storage
            .get(&TypeId::of::<Transform>())
            .unwrap()
            .downcast_ref::<SparseSet<Transform>>()
            .unwrap();

        assert_eq!(sparse_set.get(entity).is_some(), true);
    }

    #[test]
    fn check_add_component() {
        //TODO: test spawning
    }
}
