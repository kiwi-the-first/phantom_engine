use glam::*;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::ecs::component_registry::ComponentEntry;
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;

/// Pins an entity to the camera's visible rectangle so it behaves like screen-space
/// UI. The anchor system rewrites the entity's `Transform` each frame (see
/// `crate::ui::update_anchors`).
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Anchor {
    /// Normalized position in the camera rect, per axis in `[-1, 1]`:
    /// `(-1, 1)` = top-left, `(0, 0)` = center, `(1, -1)` = bottom-right.
    pub anchor: Vec2,
    /// Pixel offset from the anchor point (kept constant on screen).
    pub offset: Vec2,
    /// Size multiplier on the sprite's native pixel size; `(1, 1)` = native.
    pub base_scale: Vec2,
}

impl Reflection for Anchor {
    fn get_fields(&self) -> Vec<Field> {
        vec![
            Field::Vec2("Anchor", self.anchor),
            Field::Vec2("Offset", self.offset),
            Field::Vec2("Base Scale", self.base_scale),
        ]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        if let Some(Field::Vec2(_, anchor)) = fields.get(0) {
            self.anchor = *anchor;
        }
        if let Some(Field::Vec2(_, offset)) = fields.get(1) {
            self.offset = *offset;
        }
        if let Some(Field::Vec2(_, base_scale)) = fields.get(2) {
            self.base_scale = *base_scale;
        }
    }
}

impl Component for Anchor {
    const NAME: &'static str = "Anchor";
}

#[::ctor::ctor]
fn __register_anchor() {
    crate::ecs::component_registry::register_component(
        "Anchor",
        ComponentEntry(
            __deserialize_anchor,
            __add_default_anchor,
            __remove_anchor,
            true,
        ),
    );
}

fn __deserialize_anchor(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Anchor>>(data).unwrap())
}

fn __add_default_anchor(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Anchor::default());
    })
}

fn __remove_anchor(world: &mut World, entity: Entity) {
    world.remove_component::<Anchor>(entity);
}

impl Default for Anchor {
    fn default() -> Self {
        Self {
            anchor: Vec2::ZERO,
            offset: Vec2::ZERO,
            base_scale: Vec2::ONE,
        }
    }
}
