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
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub offset: Vec2,
    #[serde(skip)]
    pub colliding_with: Vec<Entity>,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            offset: Vec2::ZERO,
            colliding_with: Vec::new(),
        }
    }
}

impl Reflection for Collider {
    fn get_fields(&self) -> Vec<Field> {
        vec![
            Field::F32("Width", self.width),
            Field::F32("Height", self.height),
            Field::Vec2("Offset", self.offset),
        ]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        if let Some(Field::F32(_, width)) = fields.get(0) {
            self.width = *width;
        }
        if let Some(Field::F32(_, height)) = fields.get(1) {
            self.height = *height;
        }
        if let Some(Field::Vec2(_, offset)) = fields.get(2) {
            self.offset = *offset;
        }
    }
}

impl Component for Collider {
    const NAME: &'static str = "Collider";
}

#[::ctor::ctor]
fn __register_collider() {
    crate::ecs::component_registry::register_component(
        "Collider",
        ComponentEntry(
            __deserialize_collider,
            __add_default_collider,
            __remove_collider,
            true,
        ),
    );
}

fn __deserialize_collider(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Collider>>(data).unwrap())
}

fn __add_default_collider(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Collider::default());
    })
}

fn __remove_collider(world: &mut World, entity: Entity) {
    world.remove_component::<Collider>(entity);
}
