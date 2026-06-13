use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::ecs::component_registry::ComponentEntry;
use crate::reflecton::Reflection;
use crate::reflecton::asset_types::SpriteAsset;
use crate::reflecton::fields::Field;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct Sprite {
    pub asset: SpriteAsset,
}

impl Reflection for Sprite {
    fn get_fields(&self) -> Vec<Field> {
        vec![Field::Sprite("asset path", self.asset.0)]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        match fields.get(0).unwrap() {
            Field::Sprite(_name, asset_field) => self.asset = SpriteAsset(*asset_field),
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
        "Sprite",
        ComponentEntry(
            __deserialize_sprite,
            __add_default_sprite,
            __remove_sprite,
            true,
        ),
    );
}

fn __deserialize_sprite(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Sprite>>(data).unwrap())
}

fn __add_default_sprite(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Sprite::default());
    })
}

fn __remove_sprite(world: &mut World, entity: Entity) {
    world.remove_component::<Sprite>(entity);
}

impl Sprite {
    pub fn new(asset: SpriteAsset) -> Self {
        Self { asset: asset }
    }
}
