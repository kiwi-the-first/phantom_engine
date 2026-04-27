use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{
    AnyStorage, Component, Entity, SparseSet, WorldData, component_registry::COMPONENT_REGISTRY,
    components::Transform, sparse_set, world_data,
};

pub struct World {
    sparse_set_storage: HashMap<&'static str, Box<dyn AnyStorage>>,
    next_entity_id: u32,
    deleted_entity_ids: Vec<Entity>,
    generations: Vec<u32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            sparse_set_storage: HashMap::new(),
            next_entity_id: 0,
            deleted_entity_ids: Vec::new(),
            generations: Vec::new(),
        }
    }

    pub fn summon(&mut self) -> Entity {
        // Generate entity_id
        let entity = self.deleted_entity_ids.pop().unwrap_or_else(|| {
            let id = self.next_entity_id;
            self.next_entity_id = self.next_entity_id + 1;
            self.generations.push(0);
            Entity {
                id: id,
                generation: 0,
            }
        });
        // Add transform by default
        self.add_component(entity, Transform::default());

        entity
    }

    pub fn destroy(&mut self, entity: Entity) {
        for (_type_id, storage) in self.sparse_set_storage.iter_mut() {
            storage.remove(entity.id);
        }

        self.generations[entity.id as usize] += 1;
        self.deleted_entity_ids.push(Entity {
            id: entity.id,
            generation: self.generations[entity.id as usize],
        });
    }

    pub fn add_component<C: Component + serde::Serialize>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> &mut C {
        // Make sure sparse set for C exists if not create it
        self.sparse_set_storage
            .entry(C::NAME)
            .or_insert_with(|| Box::new(SparseSet::<C>::new()));

        if let Some(sparse_set) = self.sparse_set_storage.get_mut(C::NAME) {
            if let Some(sparse_set) = sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>() {
                sparse_set.insert(entity.id, component);
            }
        }

        self.get_component_mut(entity).unwrap()
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) {
        if let Some(sparse_set) = self.sparse_set_storage.get_mut(C::NAME) {
            if let Some(sparse_set) = sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>() {
                sparse_set.remove(entity.id);
            }
        }
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        if entity.generation != self.generations[entity.id as usize] {
            return None;
        }

        self.sparse_set_storage
            .get(C::NAME)
            .and_then(|sparse_set| sparse_set.as_any().downcast_ref::<SparseSet<C>>())
            .and_then(|sparse_set| sparse_set.get(entity.id))
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        if entity.generation != self.generations[entity.id as usize] {
            return None;
        }
        self.sparse_set_storage
            .get_mut(C::NAME)
            .and_then(|sparse_set| sparse_set.as_any_mut().downcast_mut::<SparseSet<C>>())
            .and_then(|sparse_set| sparse_set.get_mut(entity.id))
    }
    // pub fn has_component<C>(entity_id, component) -> bool {}

    // TODO: Change to return Vec<(u32, &C)>
    pub fn query_with<C: Component>(&self) -> Vec<Entity> {
        let sparse_set = self
            .sparse_set_storage
            .get(C::NAME)
            .and_then(|sparse_set| sparse_set.as_any().downcast_ref::<SparseSet<C>>());

        let mut entities = Vec::new();

        if let Some(sparse_set) = sparse_set {
            for entity_id in &sparse_set.entity {
                entities.push(Entity {
                    id: *entity_id,
                    generation: self.generations[*entity_id as usize],
                });
            }
        }

        entities
    }
    pub fn query_with2<A: Component, B: Component>(&self) -> Vec<Entity> {
        let entities_a = self.query_with::<A>();
        let entities_b = self.query_with::<B>();

        let result = entities_a
            .iter()
            .filter(|x| entities_b.contains(x))
            .copied()
            .collect();

        result
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut components = Vec::new();
        for (type_name, storage) in &self.sparse_set_storage {
            let name = type_name.to_string();
            let bytes = storage.serialize();
            components.push((name, bytes));
        }

        let world_data = WorldData {
            next_entity_id: self.next_entity_id,
            deleted_entity_ids: self.deleted_entity_ids.clone(),
            generations: self.generations.clone(),
            components,
        };

        bincode::serialize(&world_data).unwrap()
    }

    pub fn deserialize(data: &[u8]) -> World {
        let world_data = bincode::deserialize::<WorldData>(data).unwrap();

        let mut world = World::new();
        world.next_entity_id = world_data.next_entity_id;
        world.deleted_entity_ids = world_data.deleted_entity_ids;
        world.generations = world_data.generations;

        for (type_name, bytes) in world_data.components {
            let registry = COMPONENT_REGISTRY.get().unwrap().lock().unwrap();
            if let Some((key, deserialize_fn)) = registry.get_key_value(type_name.as_str()) {
                let storage = deserialize_fn(&bytes);
                world.sparse_set_storage.insert(key, storage);
            }
        }

        world
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;

    #[test]
    fn check_spawn_generates_transform() {
        let mut world = World::new();
        world.summon();

        assert_eq!(world.sparse_set_storage.contains_key(Transform::NAME), true);
    }

    #[test]
    fn check_spawn_generates_correct_id() {
        let mut world = World::new();
        let entity_zero = world.summon();
        let entity_one = world.summon();
        assert_eq!(entity_zero.id, 0);
        assert_eq!(entity_one.id, 1);
    }

    #[test]
    fn check_spawned_entity_has_transform() {
        let mut world = World::new();
        let entity = world.summon();
        let sparse_set = world
            .sparse_set_storage
            .get(Transform::NAME)
            .unwrap()
            .as_any()
            .downcast_ref::<SparseSet<Transform>>()
            .unwrap();

        assert_eq!(sparse_set.get(entity.id).is_some(), true);
    }

    #[test]
    fn check_add_component() {
        let mut world = World::new();
        let entity = world.summon();
        world.add_component(entity, Transform::default());
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::ZERO);
    }

    #[test]
    fn check_get_component() {
        let mut world = World::new();
        let entity = world.summon();
        world.add_component(entity, Transform::default());
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::ZERO);
    }

    #[test]
    fn check_get_component_mut() {
        let mut world = World::new();
        let entity = world.summon();
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
        let entity = world.summon();
        world.add_component(entity, Transform::default());
        assert_eq!(world.get_component::<Transform>(entity).is_some(), true);
        world.remove_component::<Transform>(entity);
        assert_eq!(world.get_component::<Transform>(entity).is_none(), true);
    }

    #[test]
    fn check_generations() {
        let mut world = World::new();
        let entity_one = world.summon();
        let entity_two = world.summon();
        //self owned data
        let entity_vec = vec![entity_one.clone(), entity_two.clone()];
        world.destroy(entity_two);
        // Check return none
        assert_eq!(world.get_component::<Transform>(entity_two), None);
        // Check stored return none
        assert_eq!(world.get_component::<Transform>(entity_vec[1]), None);
        // Check generations is as long as all max ids
        assert_eq!(world.generations.len(), 2);
    }

    #[test]
    fn check_serialize_deserialize() {
        let mut world = World::new();
        let entity = world.summon();

        let transform = world.get_component_mut::<Transform>(entity).unwrap();
        transform.position = Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let bytes = world.serialize();

        let new_world = World::deserialize(&bytes);

        let transform = new_world.get_component::<Transform>(entity).unwrap();
        assert_eq!(
            transform.position,
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }
}
