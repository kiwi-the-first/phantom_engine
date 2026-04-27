use log::*;

use crate::actions::Command;

pub struct CommandSummonEntity {}

impl Command for CommandSummonEntity {
    fn execute(&mut self) {
        debug!("EXECUTING SUMMON!")
    }

    fn undo(&mut self) {
        debug!("UNDOING SUMMON!")
    }
}
