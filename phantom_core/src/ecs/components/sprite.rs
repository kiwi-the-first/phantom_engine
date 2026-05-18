use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::ecs::entity;
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct Sprite {
    asset_path: String,
}

impl Reflection for Sprite {
    fn get_fields(&self) -> Vec<Field> {
        vec![Field::String("asset path", self.asset_path.clone())]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        match fields.get(0).unwrap() {
            Field::String(_name, asset_path_field) => {
                self.asset_path = asset_path_field.to_string()
            }
            _ => (),
        }
    }
}

impl Component for Sprite {
    const NAME: &'static str = "Sprite";
}

#[::ctor::ctor]
fn __register_sprite() {
    crate::ecs::component_registry::register_component(
        "sprite",
        __deserialize_sprite,
        __add_default_sprite,
    );
}

fn __deserialize_sprite(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(bincode::deserialize::<SparseSet<Sprite>>(data).unwrap())
}

fn __add_default_sprite(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Sprite::default());
    })
}

impl Sprite {
    pub fn new(asset_path: &'static str) -> Self {
        Self {
            asset_path: asset_path.to_string(),
        }
    }
}
