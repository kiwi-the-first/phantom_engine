use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::SparseSet;
use crate::ecs::component::Component;
use phantom_macros::component;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Component for Transform {
    const NAME: &'static str = "Transform";
}

#[::ctor::ctor]
fn __register_transform() {
    crate::ecs::component_registry::register_component("Transform", __deserialize_transform);
}

fn __deserialize_transform(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(bincode::deserialize::<SparseSet<Transform>>(data).unwrap())
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
