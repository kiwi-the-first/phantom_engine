use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{AnyStorage, Component, SparseSet, components::Transform, sparse_set};

pub struct World {
    sparse_set_storage: HashMap<TypeId, Box<dyn AnyStorage>>,
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

    pub fn destroy(&mut self, entity_id: u32) {
        for (_type_id, storage) in self.sparse_set_storage.iter_mut() {
            storage.remove(entity_id);
        }

        self.deleted_entity_ids.push(entity_id);
    }

    pub fn add_component<C: Component>(&mut self, entity_id: u32, component: C) {
        // Make sure sparse set for C exists if not create it
        self.sparse_set_storage
            .entry(TypeId::of::<C>())
            .or_insert_with(|| Box::new(SparseSet::<C>::new()));

        if let Some(sparse_set) = self.sparse_set_storage.get_mut(&TypeId::of::<C>()) {
            if let Some(sparse_set) = sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>() {
                sparse_set.insert(entity_id, component);
            }
        }
    }

    pub fn remove_component<C: Component>(&mut self, entity_id: u32) {
        if let Some(sparse_set) = self.sparse_set_storage.get_mut(&TypeId::of::<C>()) {
            if let Some(sparse_set) = sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>() {
                sparse_set.remove(entity_id);
            }
        }
    }

    pub fn get_component<C: Component>(&self, entity_id: u32) -> Option<&C> {
        self.sparse_set_storage
            .get(&TypeId::of::<C>())
            .and_then(|sparse_set| sparse_set.as_any().downcast_ref::<SparseSet<C>>())
            .and_then(|sparse_set| sparse_set.get(entity_id))
    }

    pub fn get_component_mut<C: Component>(&mut self, entity_id: u32) -> Option<&mut C> {
        self.sparse_set_storage
            .get_mut(&TypeId::of::<C>())
            .and_then(|sparse_set| sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>())
            .and_then(|sparse_set| sparse_set.get_mut(entity_id))
    }
    // pub fn has_component<C>(entity_id, component) -> bool {}

    // TODO: Change to return Vec<(u32, &C)>
    pub fn query_with<C: Component>(&self) -> Vec<u32> {
        let sparse_set = self
            .sparse_set_storage
            .get(&TypeId::of::<C>())
            .and_then(|sparse_set| sparse_set.as_any().downcast_ref::<SparseSet<C>>());

        let mut entities = Vec::new();

        if let Some(sparse_set) = sparse_set {
            for entity in &sparse_set.entity {
                entities.push(*entity);
            }
        }

        entities
    }
    // pub fn query_with2<A,B>() -> Vec<entity> {}
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

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
            .as_any()
            .downcast_ref::<SparseSet<Transform>>()
            .unwrap();

        assert_eq!(sparse_set.get(entity).is_some(), true);
    }

    #[test]
    fn check_add_component() {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default());
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::ZERO);
    }

    #[test]
    fn check_get_component() {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default());
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::ZERO);
    }

    #[test]
    fn check_get_component_mut() {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default());
        let transform = world.get_component_mut::<Transform>(entity).unwrap();
        transform.position = Vec3 {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        };
        let comparison_vec = Vec3 {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        };
        assert_eq!(transform.position, comparison_vec);
    }

    #[test]
    fn check_remove_component() {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default());
        assert_eq!(world.get_component::<Transform>(entity).is_some(), true);
        world.remove_component::<Transform>(entity);
        assert_eq!(world.get_component::<Transform>(entity).is_none(), true);
    }
}
