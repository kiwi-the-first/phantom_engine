use crate::{ecs::World, scripting::Context};

pub trait Script {
    fn start(&mut self, world: &mut World, ctx: &mut Context) {}
    fn update(&mut self, world: &mut World, ctx: &mut Context) {}
}
