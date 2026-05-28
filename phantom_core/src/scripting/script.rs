use crate::{
    ecs::{Entity, World, entity},
    scripting::ScriptContext,
};

pub trait Script {
    fn start(&mut self, entity: Entity, world: &mut World, ctx: &ScriptContext) {}
    fn update(&mut self, entity: Entity, world: &mut World, ctx: &ScriptContext) {}
}
