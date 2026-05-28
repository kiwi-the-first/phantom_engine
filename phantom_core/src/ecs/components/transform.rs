use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::ecs::component_registry::ComponentEntry;
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Reflection for Transform {
    fn get_fields(&self) -> Vec<Field> {
        vec![
            Field::Vec3("position", self.position),
            Field::TransQuat("rotation", self.rotation),
            Field::Vec3("scale", self.scale),
        ]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        match fields.get(0).unwrap() {
            Field::Vec3(_name, position_field) => self.position = *position_field,
            _ => {}
        };
        match fields.get(1).unwrap() {
            Field::TransQuat(_name, rotation_field) => self.rotation = *rotation_field,
            _ => {}
        };
        match fields.get(2).unwrap() {
            Field::Vec3(_name, scale_field) => self.scale = *scale_field,
            _ => {}
        };
    }
}

impl Component for Transform {
    const NAME: &'static str = "Transform";
}

#[::ctor::ctor]
fn __register_transform() {
    crate::ecs::component_registry::register_component(
        "Transform",
        ComponentEntry(__deserialize_transform, __add_default_transform, true),
    );
}

fn __deserialize_transform(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Transform>>(data).unwrap())
}

fn __add_default_transform(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Transform::default());
    })
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}
