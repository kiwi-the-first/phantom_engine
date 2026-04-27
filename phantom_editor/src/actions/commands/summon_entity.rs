use std::sync::{Arc, Mutex};

use log::*;

use crate::{actions::Command, context::EditorContext};

pub struct CommandSummonEntity {}

impl Command for CommandSummonEntity {
    fn execute(&mut self, ctx: &Arc<Mutex<EditorContext>>) {
        let world = &mut ctx.lock().unwrap().active_world;
        //let entity = world.summon();
    }

    fn undo(&mut self, ctx: &Arc<Mutex<EditorContext>>) {
        debug!("UNDOING SUMMON!")
    }
}
