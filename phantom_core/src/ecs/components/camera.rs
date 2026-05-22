use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;
use phantom_macros::component;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    pub zoom: f32,
    pub background_color: [u8; 4],
}

impl Reflection for Camera {
    fn get_fields(&self) -> Vec<Field> {
        vec![
            Field::F32("Zoom", self.zoom),
            Field::Color("Background Color", self.background_color),
        ]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        match fields.get(0).unwrap() {
            Field::F32(_name, zoom_field) => self.zoom = *zoom_field,
            _ => {}
        };
        match fields.get(1).unwrap() {
            Field::Color(_name, background_color) => self.background_color = *background_color,
            _ => {}
        }
    }
}

impl Component for Camera {
    const NAME: &'static str = "Camera";
}

#[::ctor::ctor]
fn __register_camera() {
    crate::ecs::component_registry::register_component(
        "Camera",
        __deserialize_camera,
        __add_default_camera,
    );
}

fn __deserialize_camera(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(bincode::deserialize::<SparseSet<Camera>>(data).unwrap())
}

fn __add_default_camera(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Camera::default());
    })
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            background_color: [3, 4, 12, 255],
        }
    }
}

impl Camera {}
