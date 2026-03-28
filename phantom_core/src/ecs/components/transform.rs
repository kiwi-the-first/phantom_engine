use glam::*;

use crate::ecs::component::Component;

pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ZERO,
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
