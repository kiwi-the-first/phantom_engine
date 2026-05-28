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
pub struct Camera {
    pub zoom: f32,
    pub background_color: [u8; 4],
    pub reference_resolution: UVec2,
}

impl Reflection for Camera {
    fn get_fields(&self) -> Vec<Field> {
        vec![
            Field::F32("Zoom", self.zoom),
            Field::Color("Background Color", self.background_color),
            Field::UVec2("Ref Resolution", self.reference_resolution),
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
        match fields.get(2).unwrap() {
            Field::UVec2(_name, reference_resolution) => {
                self.reference_resolution = *reference_resolution
            }
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
        ComponentEntry(__deserialize_camera, __add_default_camera, true),
    );
}

fn __deserialize_camera(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Camera>>(data).unwrap())
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
            reference_resolution: UVec2 { x: 1280, y: 720 },
        }
    }
}

impl Camera {}
