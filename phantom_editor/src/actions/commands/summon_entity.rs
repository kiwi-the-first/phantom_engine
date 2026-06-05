use log::*;
use phantom_core::ecs::Entity;

use crate::{actions::Command, context::EditorContext};

pub struct CommandSummonEntity {
    spawned_entity: Option<Entity>,
}

impl Command for CommandSummonEntity {
    fn execute(&mut self, ctx: &mut EditorContext) {
        let world = &mut ctx.active_world;
        if self.spawned_entity.is_none() {
            self.spawned_entity = Some(world.summon());
            trace!(
                "Executing summon entity command entity: {}",
                self.spawned_entity.unwrap()
            )
        } else {
            //Redo
            self.spawned_entity = Some(world.summon_with_id(self.spawned_entity.unwrap()));
            trace!(
                "Redoing summon entity command entity: {}",
                self.spawned_entity.unwrap()
            )
        }
    }

    fn undo(&mut self, ctx: &mut EditorContext) {
        let world = &mut ctx.active_world;
        world.destroy(self.spawned_entity.unwrap());
        trace!(
            "Undoing summon entity command entity: {}",
            self.spawned_entity.unwrap()
        )
    }
}

impl CommandSummonEntity {
    pub fn new() -> Self {
        Self {
            spawned_entity: None,
        }
    }
}
